use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, env, fmt, fs, path::PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, spanned::Spanned, Error, ItemFn, LitStr, Result, Token};

// TODO it would probably be better if this ended up in the directory of the main crate that's
// being built rater than in the out directory of the metrics-attributes-macro crate
static METRICS_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::new();
    // This is set in build.rs
    path.push(env!("OUT_DIR"));

    let compile_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("error getting system time")
        .as_micros();
    path.push(format!("metrics-{}.yaml", compile_time));

    println!("Writing list of metrics to: {}", path.display());

    path
});
const DEFAULT_METRIC_BASE_NAME: &str = "function";
const DEFAULT_PROMETHEUS_URL: &str = "http://localhost:9090";

#[derive(Default)]
struct Args {
    name: Option<String>,
    infallible: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = Args::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(Error::new(
                        input.span(),
                        "expected only a single `name` argument",
                    ))?;
                }
                let name = input.parse::<StrArg<kw::name>>()?.value;
                // TODO validate label name
                args.name = Some(name.value());
            } else if lookahead.peek(kw::infallible) {
                input.parse::<kw::infallible>()?;
                args.infallible = true;
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn autometrics(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as Args);
    let item = parse_macro_input!(item as ItemFn);

    let output = match autometrics_inner(args, item) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

fn autometrics_inner(args: Args, item: ItemFn) -> Result<TokenStream> {
    let span = item.span();
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;
    let attrs = item.attrs;

    let prometheus_url =
        env::var("PROMETHEUS_URL").unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string());

    // If the function is async we need to add a .await after the block
    let (maybe_async, maybe_await) = if sig.asyncness.is_some() {
        (quote! { async move }, quote! { .await })
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    // TODO make sure we import metrics macros from the right place
    // TODO maybe it's okay if metrics is a peer dependency
    let function_name = sig.ident.to_string();
    let base_name = if let Some(name) = &args.name {
        name.as_str()
    } else {
        DEFAULT_METRIC_BASE_NAME
    };
    let histogram_name = format!("{base_name}_duration_seconds");
    let counter_name = format!("{histogram_name}_count");

    // Write these metrics to a file
    // TODO we could be more efficient about this
    let labels = if args.infallible {
        HashMap::from([("function", vec![function_name.clone()])])
    } else {
        HashMap::from([
            ("function", vec![function_name.clone()]),
            ("result", vec!["ok".to_string(), "err".to_string()]),
        ])
    };
    let metrics = [
        Metric {
            name: counter_name.clone(),
            labels: labels.clone(),
            ty: MetricType::Counter,
        },
        Metric {
            name: histogram_name.clone(),
            labels,
            ty: MetricType::Histogram,
        },
    ];
    write_metrics_to_file(&metrics).map_err(|err| {
        Error::new(
            span,
            format!(
                "error writing to metrics file {} {}",
                METRICS_FILE.display(),
                err
            ),
        )
    })?;

    // If the function is marked as infallible, we won't add the "result" label, otherwise we will
    let track_metrics = if args.infallible {
        quote! {
            metrics::histogram!(#histogram_name, duration, "function" => #function_name);
        }
    } else {
        quote! {
            use autometrics::__private::{GetLabels, GetLabelsFromResult, str_replace};
            // Metric labels must use underscores as separators rather than colons.
            // The str_replace macro produces a static str rather than a String.
            // Note that we cannot determine the module path at macro expansion time
            // (see https://github.com/rust-lang/rust/issues/54725), only at compile/run time
            let module_path = str_replace!(module_path!(), "::", "_");

            // Note that the Rust compiler should optimize away this if/else statement because
            // it's smart enough to figure out that only one branch will ever be hit for a given function
            if let Some(label) = ret.__metrics_attributes_get_result_label() {
                metrics::histogram!(#histogram_name, duration, "function" => #function_name, "module" => module_path, "result" => label);
            } else {
                metrics::histogram!(#histogram_name, duration, "function" => #function_name, "module" => module_path);
            }
        }
    };

    // Add the metrics to the function documentation
    let function_label = format!("{{function=\"{function_name}\"}}");
    let request_rate = format!("sum by (module) (rate({counter_name}{function_label}[5m]))");
    let request_rate_doc = format!("# Rate of calls to the `{function_name}` function per second, averaged over 5 minute windows\n{request_rate}");
    let request_rate_doc = format!(
        "- [Request Rate]({})",
        make_prometheus_url(&prometheus_url, &request_rate_doc)
    );
    let error_rate_doc = if args.infallible {
        String::new()
    } else {
        let error_rate = format!("# Percentage of calls to the `{function_name}` function that return errors, averaged over 5 minute windows
sum by (module) (rate({counter_name}{{function=\"{function_name}\",result=\"err\"}}[5m])) / {request_rate}");
        format!(
            "\n- [Error Rate]({})",
            make_prometheus_url(&prometheus_url, &error_rate)
        )
    };
    let latency = format!("sum by (le, module) (rate({histogram_name}{function_label}[5m]))");
    let latency = format!(
        "# 95th and 99th percentile latencies
# (Note this will calculate the latencies if the metric is exported as a histogram)
histogram_quantile(0.99, {latency}) or
histogram_quantile(0.95, {latency}) or
# (This will show the latencies if the metric is exported as a summary)
sum by (module, quantile) rate({histogram_name}{{function=\"{function_name}\",quantile=~\"0.95|0.99\"}}[5m])"
    );
    let latency_doc = format!(
        "- [Latency (95th and 99th percentiles)]({})",
        make_prometheus_url(&prometheus_url, &latency)
    );
    let docs = format!(
        "\n\n## Metrics

View the live metrics for this function:
{request_rate_doc}{error_rate_doc}
{latency_doc}

This function has the following metrics associated with it:
- `{counter_name}{{function=\"{function_name}\"}}`
- `{histogram_name}{{function=\"{function_name}\"}}`",
    );

    Ok(quote! {
        #(#attrs)*
        #[doc=#docs]
        #vis #sig {
            let __autometrics_start = ::std::time::Instant::now();

            let ret = #maybe_async { #block } #maybe_await;

            let duration = __autometrics_start.elapsed().as_secs_f64();
            #track_metrics

            ret
        }
    })
}

fn make_prometheus_url(url: &str, query: &str) -> String {
    let mut url = url.to_string();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("graph?g0.expr=");
    url.push_str(&urlencoding::encode(query));
    url.push_str("&g0.tab=0");
    url
}

// Copied from tracing-attributes
struct StrArg<T> {
    value: LitStr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for StrArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(infallible);
}

#[derive(Serialize)]
struct Metric {
    name: String,
    labels: HashMap<&'static str, Vec<String>>,
    ty: MetricType,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum MetricType {
    Histogram,
    Counter,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::Counter => f.write_str("counter"),
            MetricType::Histogram => f.write_str("histogram"),
        }
    }
}

// TODO figure out a better file format
// TODO figure out how to combine duplicate rows efficiently (i.e. after we're done appending to this file)
fn write_metrics_to_file(metrics: &[Metric]) -> std::result::Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&*METRICS_FILE)
        .map_err(|err| format!("error opening file: {:?}", err))?;
    serde_yaml::to_writer(&mut file, metrics)
        .map_err(|err| format!("error writing metric to file: {:?}", err))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_fn() {
        let item = quote! {
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        };
        let item: ItemFn = syn::parse2(item).unwrap();
        let actual = autometrics_inner(Default::default(), item).unwrap();
        let expected = quote! {
            pub fn add(a: i32, b: i32) -> i32 {
                let __start_internal = ::std::time::Instant::now();

                let ret = {
                    a + b
                };

                ::metrics::histogram!("add_duration_seconds", __start_internal.elapsed().as_secs_f64());
                ::metrics::increment_counter!("add_total");

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn async_fn() {
        let item = quote! {
            async fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        };
        let item: ItemFn = syn::parse2(item).unwrap();
        let actual = autometrics_inner(Default::default(), item).unwrap();
        let expected = quote! {
            async fn add(a: i32, b: i32) -> i32 {
                let __start_internal = ::std::time::Instant::now();

                let ret = {
                    a + b
                }.await;

                ::metrics::histogram!("add_duration_seconds", __start_internal.elapsed().as_secs_f64());
                ::metrics::increment_counter!("add_total");

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn return_result() {
        let item = quote! {
            fn check_positive(num: i32) -> Result<(), ()> {
                if num >= 0 {
                    Ok(())
                } else {
                    Err(())
                }
            }
        };
        let item: ItemFn = syn::parse2(item).unwrap();
        let actual = autometrics_inner(Default::default(), item).unwrap();
        let expected = quote! {
            fn check_positive(num: i32) -> Result<(), ()> {
                let __start_internal = ::std::time::Instant::now();

                let ret = {
                    if num >= 0 {
                        Ok(())
                    } else {
                        Err(())
                    }
                };

                let status = if ret.is_ok() {
                    "ok"
                } else {
                    "err"
                };
                ::metrics::histogram!("check_positive_duration_seconds", "result" => status, __start_internal.elapsed().as_secs_f64());
                ::metrics::increment_counter!("check_positive_total", "result" => status);

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
