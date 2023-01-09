use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, fmt, fs, path::PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, spanned::Spanned, Error, Expr, ItemFn, LitStr, Result, Token};

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
const DEFAULT_METRIC_BASE_NAME: &str = "function_call";

#[derive(Default)]
struct InstrumentArgs {
    name: Option<String>,
    infallible: bool,
}

impl Parse for InstrumentArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = InstrumentArgs::default();
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
pub fn instrument(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as InstrumentArgs);
    let item = parse_macro_input!(item as ItemFn);

    let output = match instrument_inner(args, item) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

fn instrument_inner(args: InstrumentArgs, item: ItemFn) -> Result<TokenStream> {
    let span = item.span();
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;
    let attrs = item.attrs;

    // If the function is async we need to add a .await after the block
    let maybe_await = if sig.asyncness.is_some() {
        quote! { .await }
    } else {
        TokenStream::new()
    };

    // TODO make sure we import metrics macros from the right place
    // TODO maybe it's okay if metrics is a peer dependency
    let function_name = sig.ident.to_string();
    let base_name = if let Some(name) = &args.name {
        name.as_str()
    } else {
        DEFAULT_METRIC_BASE_NAME
    };
    let counter_name = format!("{}_total", base_name);
    let histogram_name = format!("{}_duration_seconds", base_name);

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
            metrics::increment_counter!(#counter_name, "function" => #function_name);
        }
    } else {
        quote! {
            use metrics_attributes::__private::{GetLabels, GetLabelsFromResult, str_replace};
            let module_path = str_replace!(module_path!(), "::", "_");

            // Note that the Rust compiler should optimize away this if/else statement because
            // it's smart enough to figure out that only one branch will ever be hit for a given function
            if let Some(label) = ret.__metrics_attributes_get_result_label() {
                metrics::histogram!(#histogram_name, duration, "function" => #function_name, "module" => module_path, "result" => label);
                metrics::increment_counter!(#counter_name, "function" => #function_name, "module" => module_path, "result" => label);
            } else {
                metrics::histogram!(#histogram_name, duration, "function" => #function_name, "module" => module_path);
                metrics::increment_counter!(#counter_name, "function" => #function_name, "module" => module_path);
            }
        }
    };

    // Add the metrics to the function documentation
    let docs = format!(
        "

# Metrics

This function has the following metrics associated with it:
- `{}{{function=\"{}\"}}`
- `{}{{function=\"{}\"}}`",
        histogram_name, function_name, counter_name, function_name
    );

    // TODO generate doc comments that describe the related metrics

    Ok(quote! {
        #(#attrs)*
        #[doc=#docs]
        #vis #sig {
            let __metrics_attributes_start = ::std::time::Instant::now();

            let ret = #block #maybe_await;

            let duration = __metrics_attributes_start.elapsed().as_secs_f64();
            #track_metrics

            ret
        }
    })
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

struct ExprArg<T> {
    value: Expr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for ExprArg<T> {
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
        let actual = instrument_inner(Default::default(), item).unwrap();
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
        let actual = instrument_inner(Default::default(), item).unwrap();
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
        let actual = instrument_inner(Default::default(), item).unwrap();
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
