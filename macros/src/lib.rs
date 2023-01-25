use crate::parse::Args;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use proc_macro2::TokenStream;
use quote::quote;
use std::env;
use syn::{parse_macro_input, ItemFn, Result};

mod parse;

const DEFAULT_METRIC_BASE_NAME: &str = "function";
const DEFAULT_PROMETHEUS_URL: &str = "http://localhost:9090";

/// # Autometrics
///
/// Autometrics instruments your functions with automatically generated metrics
/// and writes Prometheus queries for you, making it easy for you to observe and
/// understand how your system performs in production.
///
/// By default, Autometrics uses a counter, histogram, and a gauge to track
/// the request rate, error rate, latency, and concurrent calls to each instrumented function.
///
/// For all of the generated metrics, Autometrics attaches the following labels:
/// - `function` - the name of the function
/// - `module` - the module path of the function (with `::` replaced by `_`)
///
/// For the function call counter, Autometrics attaches these additional labels:
/// - `result` - if the function returns a `Result`, this will either be `ok` or `error`
/// - `caller` - the name of the (autometrics-instrumented) function that called the current function
/// - (optional) `ok`/`error` - if the inner type implements `Into<&'static str>`, that value will be used as this label's value
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
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;
    let attrs = item.attrs;
    let function_name = sig.ident.to_string();

    // The PROMETHEUS_URL can be configured by passing the environment variable during build time
    let prometheus_url =
        env::var("PROMETHEUS_URL").unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string());

    // Set up metric names
    let base_name = if let Some(name) = &args.name {
        name.as_str()
    } else {
        DEFAULT_METRIC_BASE_NAME
    };
    let counter_name = format!("{base_name}.calls.count");
    let histogram_name = format!("{base_name}.calls.duration");
    let gauge_name = format!("{base_name}.calls.concurrent");

    // Build the documentation we'll add to the function's RustDocs
    let metrics_docs = create_metrics_docs(
        &prometheus_url,
        &counter_name,
        &histogram_name,
        &gauge_name,
        &function_name,
    );

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

    Ok(quote! {
        #(#attrs)*

        // Append the metrics documentation to the end of the function's documentation
        #[doc=#metrics_docs]

        #vis #sig {
            // Time the function
            let __autometrics_start = ::std::time::Instant::now();

            // Track the number of concurrent requests (the gauge will be decremented when this value
            // is dropped at the end of the function)
            let __autometrics_concurrency_tracker = {
                use autometrics::__private::{create_labels, create_concurrency_tracker, str_replace};

                // Note that we cannot determine the module path at macro expansion time
                // (see https://github.com/rust-lang/rust/issues/54725), only at compile/run time
                let module_label = str_replace!(module_path!(), "::", ".");
                let labels = create_labels(#function_name, module_label);

                // This increments a gauge and decrements the gauge again when the return value is dropped
                create_concurrency_tracker(#gauge_name, labels)
            };

            let ret = #call_function;

            // Track the number of function calls
            {
                use autometrics::__private::{
                    create_labels, register_counter, register_histogram, str_replace, Context, GetLabels,
                    GetLabelsFromResult, CALLER,
                };

                let module_label = str_replace!(module_path!(), "::", ".");
                let context = Context::current();
                let duration = __autometrics_start.elapsed().as_secs_f64();

                // Track the function calls
                let counter_labels = ret.__autometrics_get_labels(#function_name, module_label, CALLER.get());
                let counter = register_counter(#counter_name);
                counter.add(&context, 1.0, &counter_labels);

                // Track the latency
                // Histograms are more expensive than counters because they track every bucket
                // for every label combination, so we only use the function name and module labels for it
                let histogram_labels = create_labels(#function_name, module_label);
                let histogram = register_histogram(#histogram_name);
                histogram.record(&context, duration, &histogram_labels);
            }

            ret
        }
    })
}

/// Create Prometheus queries for the generated metric and
/// package them up into a RustDoc string
fn create_metrics_docs(
    prometheus_url: &str,
    counter_name: &str,
    histogram_name: &str,
    gauge_name: &str,
    function_name: &str,
) -> String {
    let counter_name = to_prometheus_string(counter_name);
    let gauge_name = to_prometheus_string(gauge_name);
    let bucket_name = format!("{}_bucket", to_prometheus_string(histogram_name));

    let request_rate = request_rate_query(&counter_name, "function", &function_name);
    let request_rate_url = make_prometheus_url(&prometheus_url, &request_rate, &format!("Rate of calls to the `{function_name}` function per second, averaged over 5 minute windows"));
    let callee_request_rate = request_rate_query(&counter_name, "caller", &function_name);
    let callee_request_rate_url = make_prometheus_url(&prometheus_url, &callee_request_rate, &format!("Rate of calls to functions called by `{function_name}` per second, averaged over 5 minute windows"));

    let error_ratio = &error_ratio_query(&counter_name, "function", &function_name);
    let error_ratio_url = make_prometheus_url(&prometheus_url, &error_ratio, &format!("Percentage of calls to the `{function_name}` function that return errors, averaged over 5 minute windows"));
    let callee_error_ratio = &error_ratio_query(&counter_name, "caller", &function_name);
    let callee_error_ratio_url = make_prometheus_url(&prometheus_url, &callee_error_ratio, &format!("Percentage of calls to functions called by `{function_name}` that return errors, averaged over 5 minute windows"));

    let latency = latency_query(&bucket_name, "function", &function_name);
    let latency_url = make_prometheus_url(
        &prometheus_url,
        &latency,
        &format!("95th and 99th percentile latencies for the `{function_name}` function"),
    );

    let concurrent_calls = concurrent_calls_query(&gauge_name, "function", &function_name);
    let concurrent_calls_url = make_prometheus_url(
        &prometheus_url,
        &concurrent_calls,
        &format!("Concurrent calls to the `{function_name}` function"),
    );

    format!(
        "\n\n---

## Autometrics

View the live metrics for the `{function_name}` function:
- [Request Rate]({request_rate_url})
- [Error Ratio]({error_ratio_url})
- [Latency (95th and 99th percentiles)]({latency_url})
- [Concurrent Calls]({concurrent_calls_url})

Or, dig into the metrics of *functions called by* `{function_name}`:
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

fn to_prometheus_string(string: &str) -> String {
    string
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn request_rate_query(counter_name: &str, label_key: &str, label_value: &str) -> String {
    format!("sum by (function, module) (rate({counter_name}{{{label_key}=\"{label_value}\"}}[5m]))")
}

fn error_ratio_query(counter_name: &str, label_key: &str, label_value: &str) -> String {
    let request_rate = request_rate_query(counter_name, label_key, label_value);
    format!("sum by (function, module) (rate({counter_name}{{{label_key}=\"{label_value}\",result=\"err\"}}[5m])) /
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
