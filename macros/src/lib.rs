use crate::parse::{Args, Item};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::env;
use syn::{parse_macro_input, ImplItem, ItemFn, ItemImpl, Result};

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
/// ### `alerts`
///
/// **Only available when the `alerts` feature is enabled.**
///
/// Example:
/// ```rust
/// #[autometrics(alerts(success_rate = 99.9%, latency(99.9% < 200ms)))]
/// ```
///
/// The alerts feature can be used to have autometrics generate Prometheus AlertManager alerts.
/// You can specify the `success_rate` and/or `latency` target and percentile for the given function.
///
/// Add these options **only** to 1-3 top-level functions you want to generate alerts for.
/// These should be functions like the main HTTP or WebSocket handler.
/// You almost definitely do not want to be alerted for every function.
///
/// ⚠️ **Note about `latency` alerts**: The latency target **MUST** match one of the buckets
/// configured for your histogram. For example, if you want to enforce that a certain percentage of calls
/// are handled within 200ms, you must have a histogram bucket for 0.2 seconds. If there is no
/// such bucket, the alert will never fire.
#[proc_macro_attribute]
pub fn autometrics(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as parse::Args);
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

/// Add autometrics instrumentation to a single function
fn instrument_function(args: &Args, item: ItemFn) -> Result<TokenStream> {
    let track_concurrency = args.track_concurrency;
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;
    let attrs = item.attrs;
    let function_name = sig.ident.to_string();

    // The PROMETHEUS_URL can be configured by passing the environment variable during build time
    let prometheus_url =
        env::var("PROMETHEUS_URL").unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string());

    // Build the documentation we'll add to the function's RustDocs
    let metrics_docs = create_metrics_docs(&prometheus_url, &function_name, track_concurrency);

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

    #[cfg(feature = "alerts")]
    let alert_definition = if let Some(alerts) = &args.alerts {
        let function_name_uppercase = format_ident!("AUTOMETRICS_{}", function_name.to_uppercase());
        let success_rate = if let Some(success_rate) = alerts.success_rate {
            let success_rate = success_rate.normalize().to_string();
            quote! { Some(#success_rate) }
        } else {
            quote! { None }
        };
        let latency = if let Some(latency) = &alerts.latency {
            let latency_target = latency.target_seconds.normalize().to_string();
            let latency_percentile = latency.percentile.normalize().to_string();
            quote! { Some((#latency_target, #latency_percentile)) }
        } else {
            quote! { None }
        };

        quote! {
            {
                use autometrics::__private::{distributed_slice, Alert, METRICS};

                // This is a bit nuts. For every function that has alert
                // definition defined, we create a static record in a
                // distributed slice that is "gathered into a contiguous section
                // of the binary by the linker". We then iterate over this list of
                // instrumented functions to generate the alerts.
                // See https://github.com/dtolnay/linkme for how this "shenanigans" works
                #[distributed_slice(METRICS)]
                static #function_name_uppercase: Alert = Alert {
                    function: #function_name,
                    module: module_label,
                    success_rate: #success_rate,
                    latency: #latency,
                };
            }
        }
    } else {
        TokenStream::new()
    };
    #[cfg(not(feature = "alerts"))]
    let alert_definition = TokenStream::new();

    Ok(quote! {
        #(#attrs)*

        // Append the metrics documentation to the end of the function's documentation
        #[doc=#metrics_docs]

        #vis #sig {
            let __autometrics_tracker = {
                use autometrics::__private::{AutometricsTracker, TrackMetrics, str_replace};

                // Note that we cannot determine the module path at macro expansion time
                // (see https://github.com/rust-lang/rust/issues/54725), only at compile/run time
                const module_label: &'static str = str_replace!(module_path!(), "::", ".");

                #alert_definition

                AutometricsTracker::start(#function_name, module_label, #track_concurrency)
            };

            let result = #call_function;

            {
                use autometrics::__private::{CALLER, GetLabels, GetLabelsFromResult, TrackMetrics};
                let counter_labels = (&result).__autometrics_get_labels(__autometrics_tracker.function(), __autometrics_tracker.module(), CALLER.get());
                __autometrics_tracker.finish(&counter_labels);
            }

            result
        }
    })
}

/// Add autometrics instrumentation to an entire impl block
fn instrument_impl_block(args: &Args, mut item: ItemImpl) -> Result<TokenStream> {
    // Replace all of the method items in place
    item.items = item
        .items
        .into_iter()
        .map(|item| match item {
            ImplItem::Method(method) => {
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
        &format!("95th and 99th percentile latencies for the `{function}` function"),
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
