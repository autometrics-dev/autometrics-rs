use crate::parse::{AutometricsArgs, Item};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::env;
use syn::{parse_macro_input, DeriveInput, ImplItem, ItemFn, ItemImpl, Result, Data, DataEnum, Attribute, Meta, NestedMeta, Lit};

mod parse;

const COUNTER_NAME_PROMETHEUS: &str = "function_calls_count";
const HISTOGRAM_BUCKET_NAME_PROMETHEUS: &str = "function_calls_duration_bucket";
const GAUGE_NAME_PROMETHEUS: &str = "function_calls_concurrent";

const DEFAULT_PROMETHEUS_URL: &str = "http://localhost:9090";

/// # Autometrics
///
/// Autometrics instruments your functions with automatically generated metrics
/// and writes Prometheus queries for you, making it easy for you to observe and
/// understand how your system performs in production.
///
/// By default, Autometrics uses a counter and a histogram to track
/// the request rate, error rate, latency of calls to your functions.
///
/// For all of the generated metrics, Autometrics attaches the following labels:
/// - `function` - the name of the function
/// - `module` - the module path of the function (with `::` replaced by `.`)
///
/// For the function call counter, Autometrics attaches these additional labels:
/// - `result` - if the function returns a `Result`, this will either be `ok` or `error`
/// - `caller` - the name of the (autometrics-instrumented) function that called the current function
/// - (optional) `ok`/`error` - if the inner type implements `Into<&'static str>`, that value will be used as this label's value
///
/// ## Optional Parameters
///
/// ### `ok_if` and `error_if`
///
/// Example:
/// ```rust
/// #[autometrics(ok_if = Option::is_some)]
/// ```
///
/// If the function does not return a `Result`, you can use `ok_if` and `error_if` to specify
/// whether the function call was "successful" or not, as far as the metrics are concerned.
///
/// For example, if a function returns an HTTP response, you can use `ok_if` or `error_if` to
/// add the `result` label based on the status code:
/// ```rust
/// #[autometrics(ok_if = |res: &http::Response<_>| res.status().is_success())]
/// pub async fn my_handler(req: http::Request<hyper::Body>) -> http::Response<hyper::Body> {
///    // ...
/// }
/// ```
///
/// Note that the function must be callable as `f(&T) -> bool`, where `T` is the return type
/// of the instrumented function.
///
/// ### `track_concurrency`
///
/// Example:
/// ```rust
/// #[autometrics(track_concurrency)]
/// ```
///
/// Pass this argument to track the number of concurrent calls to the function (using a gauge).
/// This may be most useful for top-level functions such as the main HTTP handler that
/// passes requests off to other functions.
///
/// ### `objective`
///
/// Example:
/// ```rust
/// use autometrics::{autometrics, objectives::*};
///
/// const API_SLO: Objective = Objective::new("api")
///     .success_rate(ObjectivePercentile::P99_9)
///
/// #[autometrics(objective = API_SLO)]
/// pub fn handler() {
///    // ...
/// }
/// ```
///
/// Include this function's metrics in the specified objective or SLO.
///
/// See the docs for [Objective](https://docs.rs/autometrics/latest/autometrics/objectives/struct.Objective.html) for details on how to create objectives.
///
/// ## Instrumenting `impl` blocks
///
/// In addition to instrumenting functions, you can also instrument `impl` blocks.
///
/// Example:
/// ```rust
/// struct MyStruct;
///
/// #[autometrics]
/// impl MyStruct {
///     #[skip_autometrics]
///     pub fn new() -> Self {
///        Self
///     }
///
///     fn my_function(&self) {
///        // ...
///    }
/// }
/// ```
///
/// This will instrument all functions in the `impl` block, except for those that have the `skip_autometrics` attribute.
///
#[proc_macro_attribute]
pub fn autometrics(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as parse::AutometricsArgs);
    let item = parse_macro_input!(item as Item);

    let result = match item {
        Item::Function(item) => instrument_function(&args, item),
        Item::Impl(item) => instrument_impl_block(&args, item),
    };

    let output = match result {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

#[proc_macro_derive(LabelValues, attributes(autometrics))]
pub fn derive_label_values(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let result = derive_label_values_impl(input);
    let output = match result {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

/// Add autometrics instrumentation to a single function
fn instrument_function(args: &AutometricsArgs, item: ItemFn) -> Result<TokenStream> {
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;
    let attrs = item.attrs;
    let function_name = sig.ident.to_string();

    // The PROMETHEUS_URL can be configured by passing the environment variable during build time
    let prometheus_url =
        env::var("PROMETHEUS_URL").unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string());

    // Build the documentation we'll add to the function's RustDocs
    let metrics_docs = create_metrics_docs(&prometheus_url, &function_name, args.track_concurrency);

    // Wrap the body of the original function, using a slightly different approach based on whether the function is async
    let call_function = if sig.asyncness.is_some() {
        quote! {
            autometrics::__private::CALLER.scope(#function_name, async move {
                #block
            }).await
        }
    } else {
        quote! {
            autometrics::__private::CALLER.sync_scope(#function_name, move || {
                #block
            })
        }
    };

    let objective = if let Some(objective) = &args.objective {
        quote! { Some(#objective) }
    } else {
        quote! { None }
    };

    let counter_labels = if args.ok_if.is_some() || args.error_if.is_some() {
        // Apply the predicate to determine whether to consider the result as "ok" or "error"
        let result_label = if let Some(ok_if) = &args.ok_if {
            quote! { if #ok_if (&result) { "ok" } else { "error" } }
        } else if let Some(error_if) = &args.error_if {
            quote! { if #error_if (&result) { "error" } else { "ok" } }
        } else {
            unreachable!()
        };
        quote! {
            {
                use autometrics::__private::{CALLER, CounterLabels, GetLabelValue};
                let result_label = #result_label;
                let value_type = (&result).get_label_value();
                CounterLabels::new(
                    #function_name,
                     module_path!(),
                     CALLER.get(),
                    Some((result_label, value_type)),
                    #objective,
                )
            }
        }
    } else {
        // This will use the traits defined in the `labels` module to determine if
        // the return value was a `Result` and, if so, assign the appropriate labels
        quote! {
            {
                use autometrics::__private::{CALLER, CounterLabels, GetLabels, GetLabelsFromResult};
                let result_labels = (&result).__autometrics_get_labels();
                CounterLabels::new(
                    #function_name,
                    module_path!(),
                    CALLER.get(),
                    result_labels,
                    #objective,
                )
            }
        }
    };

    let gauge_labels = if args.track_concurrency {
        quote! { Some(&autometrics::__private::GaugeLabels { function: #function_name, module: module_path!() }) }
    } else {
        quote! { None }
    };

    Ok(quote! {
        #(#attrs)*

        // Append the metrics documentation to the end of the function's documentation
        #[doc=#metrics_docs]

        #vis #sig {
            let __autometrics_tracker = {
                use autometrics::__private::{AutometricsTracker, TrackMetrics};
                AutometricsTracker::start(#gauge_labels)
            };

            let result = #call_function;

            {
                use autometrics::__private::{HistogramLabels, TrackMetrics};
                let counter_labels = #counter_labels;
                let histogram_labels = HistogramLabels::new(
                    #function_name,
                     module_path!(),
                     #objective,
                );
                __autometrics_tracker.finish(&counter_labels, &histogram_labels);
            }

            result
        }
    })
}

/// Add autometrics instrumentation to an entire impl block
fn instrument_impl_block(args: &AutometricsArgs, mut item: ItemImpl) -> Result<TokenStream> {
    // Replace all of the method items in place
    item.items = item
        .items
        .into_iter()
        .map(|item| match item {
            ImplItem::Method(mut method) => {
                // Skip any methods that have the #[skip_autometrics] attribute
                if method
                    .attrs
                    .iter()
                    .any(|attr| attr.path.is_ident("skip_autometrics"))
                {
                    method
                        .attrs
                        .retain(|attr| !attr.path.is_ident("skip_autometrics"));
                    return ImplItem::Method(method);
                }

                let item_fn = ItemFn {
                    attrs: method.attrs,
                    vis: method.vis,
                    sig: method.sig,
                    block: Box::new(method.block),
                };
                let tokens = match instrument_function(args, item_fn) {
                    Ok(tokens) => tokens,
                    Err(err) => err.to_compile_error(),
                };
                ImplItem::Verbatim(tokens)
            }
            _ => item,
        })
        .collect();

    Ok(quote! { #item })
}

/// Create Prometheus queries for the generated metric and
/// package them up into a RustDoc string
fn create_metrics_docs(prometheus_url: &str, function: &str, track_concurrency: bool) -> String {
    let request_rate = request_rate_query(&COUNTER_NAME_PROMETHEUS, "function", &function);
    let request_rate_url = make_prometheus_url(
        &prometheus_url,
        &request_rate,
        &format!(
            "Rate of calls to the `{function}` function per second, averaged over 5 minute windows"
        ),
    );
    let callee_request_rate = request_rate_query(&COUNTER_NAME_PROMETHEUS, "caller", &function);
    let callee_request_rate_url = make_prometheus_url(&prometheus_url, &callee_request_rate, &format!("Rate of calls to functions called by `{function}` per second, averaged over 5 minute windows"));

    let error_ratio = &error_ratio_query(&COUNTER_NAME_PROMETHEUS, "function", &function);
    let error_ratio_url = make_prometheus_url(&prometheus_url, &error_ratio, &format!("Percentage of calls to the `{function}` function that return errors, averaged over 5 minute windows"));
    let callee_error_ratio = &error_ratio_query(&COUNTER_NAME_PROMETHEUS, "caller", &function);
    let callee_error_ratio_url = make_prometheus_url(&prometheus_url, &callee_error_ratio, &format!("Percentage of calls to functions called by `{function}` that return errors, averaged over 5 minute windows"));

    let latency = latency_query(&HISTOGRAM_BUCKET_NAME_PROMETHEUS, "function", &function);
    let latency_url = make_prometheus_url(
        &prometheus_url,
        &latency,
        &format!("95th and 99th percentile latencies (in seconds) for the `{function}` function"),
    );

    // Only include the concurrent calls query if the user has enabled it for this function
    let concurrent_calls_doc = if track_concurrency {
        let concurrent_calls =
            concurrent_calls_query(&GAUGE_NAME_PROMETHEUS, "function", &function);
        let concurrent_calls_url = make_prometheus_url(
            &prometheus_url,
            &concurrent_calls,
            &format!("Concurrent calls to the `{function}` function"),
        );
        format!("\n- [Concurrent Calls]({concurrent_calls_url}")
    } else {
        String::new()
    };

    format!(
        "\n\n---

## Autometrics

View the live metrics for the `{function}` function:
- [Request Rate]({request_rate_url})
- [Error Ratio]({error_ratio_url})
- [Latency (95th and 99th percentiles)]({latency_url}){concurrent_calls_doc}

Or, dig into the metrics of *functions called by* `{function}`:
- [Request Rate]({callee_request_rate_url})
- [Error Ratio]({callee_error_ratio_url})
"
    )
}

fn make_prometheus_url(url: &str, query: &str, comment: &str) -> String {
    let mut url = url.to_string();
    let comment_and_query = format!("# {comment}\n\n{query}");
    let query = utf8_percent_encode(&comment_and_query, NON_ALPHANUMERIC).to_string();

    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("graph?g0.expr=");
    url.push_str(&query);
    // Go straight to the graph tab
    url.push_str("&g0.tab=0");
    url
}

fn request_rate_query(counter_name: &str, label_key: &str, label_value: &str) -> String {
    format!("sum by (function, module) (rate({counter_name}{{{label_key}=\"{label_value}\"}}[5m]))")
}

fn error_ratio_query(counter_name: &str, label_key: &str, label_value: &str) -> String {
    let request_rate = request_rate_query(counter_name, label_key, label_value);
    format!("sum by (function, module) (rate({counter_name}{{{label_key}=\"{label_value}\",result=\"error\"}}[5m])) /
{request_rate}", )
}

fn latency_query(bucket_name: &str, label_key: &str, label_value: &str) -> String {
    let latency = format!(
        "sum by (le, function, module) (rate({bucket_name}{{{label_key}=\"{label_value}\"}}[5m]))"
    );
    format!(
        "histogram_quantile(0.99, {latency}) or
histogram_quantile(0.95, {latency})"
    )
}

fn concurrent_calls_query(gauge_name: &str, label_key: &str, label_value: &str) -> String {
    format!("sum by (function, module) {gauge_name}{{{label_key}=\"{label_value}\"}}")
}

fn derive_label_values_impl(input: DeriveInput) -> Result<TokenStream> {
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Err(syn::Error::new_spanned(input, "#[derive(LabelValues}] is only supported for enums"));
        },
    };

    let match_arms = variants
        .into_iter()
        .map(|variant| {
            let attrs: Vec<_> = variant.attrs.iter().filter(|attr| attr.path.is_ident("autometrics")).collect();

            let value_from_attr = match attrs.len() {
                0 => None,
                1 => get_label_value_attr(attrs[0])?,
                _ => {
                    let mut error =
                        syn::Error::new_spanned(attrs[1], "redundant `autometrics(label_value)` attribute");
                    error.combine(syn::Error::new_spanned(attrs[0], "note: first one here"));
                    return Err(error);
                }
            };

            let ident = variant.ident;
            let value = value_from_attr.unwrap_or_else(|| ident.clone());
            let value = value.to_string();
            Ok(quote! {
                Self::#ident => Some(#value),
            })
        })
        .collect::<Result<TokenStream>>()?;

    let ident = input.ident;
    Ok(quote! {
        #[automatically_derived]
        impl GetLabelValue for #ident {
            fn get_label_value(&self) -> Option<&'static str> {
                match self {
                    #match_arms
                }
            }
        }
    })
}

fn get_label_value_attr(attr: &Attribute) -> Result<Option<Ident>> {
    let meta = attr.parse_meta()?;
    let meta_list = match meta {
        Meta::List(list) => list,
        _ => return Err(syn::Error::new_spanned(meta, "expected a list-style attribute")),
    };

    let nested = match meta_list.nested.len() {
        // `#[autometrics()]` without any arguments is a no-op
        0 => return Ok(None),
        1 => &meta_list.nested[0],
        _ => {
            return Err(syn::Error::new_spanned(
                meta_list.nested,
                "currently only a single autometrics attribute is supported",
            ));
        }
    };

    let label_value = match nested {
        NestedMeta::Meta(Meta::NameValue(nv)) => nv,
        _ => return Err(syn::Error::new_spanned(nested, "expected `label_value = \"<value>\"`")),
    };

    if !label_value.path.is_ident("label_value") {
        return Err(syn::Error::new_spanned(
            &label_value.path,
            "unsupported autometrics attribute, expected `label_value`",
        ));
    }

    match &label_value.lit {
        Lit::Str(s) => syn::parse_str(&s.value()).map_err(|e| syn::Error::new_spanned(s, e)),
        lit => Err(syn::Error::new_spanned(lit, "expected string literal")),
    }
}
