use crate::parse::Args;
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
/// By default, Autometrics uses a histogram and a gauge to track
/// the request rate, error rate, latency, and concurrent calls to each instrumented function.
///
/// It attaches the following labels:
/// - `function` - the name of the function
/// - `module` - the module path of the function (with `::` replaced by `_`)
/// - `result` - if the function returns a `Result`, this will either be `ok` or `error`
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
    // If the function is async we need to add a .await after the block
    let (maybe_async, maybe_await) = if sig.asyncness.is_some() {
        (quote! { async move }, quote! { .await })
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    // The PROMETHEUS_URL can be configured by passing the environment variable during build time
    let prometheus_url =
        env::var("PROMETHEUS_URL").unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string());

    let base_name = if let Some(name) = &args.name {
        name.as_str()
    } else {
        DEFAULT_METRIC_BASE_NAME
    };
    let histogram_name = format!("{base_name}_duration_seconds");
    let gauge_name = format!("{base_name}_concurrent_calls");

    let setup = quote! {
        let __autometrics_start = ::std::time::Instant::now();

        let __autometrics_concurrency_tracker = {
            use autometrics::__private::{create_labels, create_concurrency_tracker, str_replace};

            // Metric labels must use underscores as separators rather than colons.
            // The str_replace macro produces a static str rather than a String.
            // Note that we cannot determine the module path at macro expansion time
            // (see https://github.com/rust-lang/rust/issues/54725), only at compile/run time
            let module_label = str_replace!(module_path!(), "::", "_");
            let labels = create_labels(#function_name, module_label);

            // This increments a gauge and decrements the gauge again when the return value is dropped
            create_concurrency_tracker(#gauge_name, labels)
        };
    };

    let track_metrics = quote! {
        {
            use autometrics::__private::{Context, GetLabels, GetLabelsFromResult, register_histogram, str_replace};

            let module_label = str_replace!(module_path!(), "::", "_");
            let labels = ret.__autometrics_get_labels(#function_name, module_label);
            let histogram = register_histogram(#histogram_name);
            let duration = __autometrics_start.elapsed().as_secs_f64();
            histogram.record(&Context::current(), duration, &labels);
        }
    };

    let metrics_docs = create_metrics_docs(
        &prometheus_url,
        &histogram_name,
        &gauge_name,
        &function_name,
    );

    Ok(quote! {
        #(#attrs)*

        // Append the metrics documentation to the end of the function's documentation
        #[doc=#metrics_docs]

        #vis #sig {
            #setup

            let ret = #maybe_async { #block } #maybe_await;

            #track_metrics

            ret
        }
    })
}

/// Create Prometheus queries for the generated metric and
/// package them up into a RustDoc string
fn create_metrics_docs(
    prometheus_url: &str,
    histogram_name: &str,
    gauge_name: &str,
    function_name: &str,
) -> String {
    let counter_name = format!("{histogram_name}_count");
    let bucket_name = format!("{histogram_name}_bucket");
    let function_label = format!("{{function=\"{function_name}\"}}");

    // Request rate
    let request_rate =
        format!("sum by (function, module) (rate({counter_name}{function_label}[5m]))");
    let request_rate_doc = format!("# Rate of calls to the `{function_name}` function per second, averaged over 5 minute windows

{request_rate}");
    let request_rate_doc = format!(
        "- [Request Rate]({})",
        make_prometheus_url(&prometheus_url, &request_rate_doc)
    );

    // Error rate
    let error_rate = format!("# Percentage of calls to the `{function_name}` function that return errors, averaged over 5 minute windows

sum by (function, module) (rate({counter_name}{{function=\"{function_name}\",result=\"err\"}}[5m])) / {request_rate}");
    let error_rate_doc = format!(
        "- [Error Rate]({})",
        make_prometheus_url(&prometheus_url, &error_rate)
    );

    // Latency
    let latency =
        format!("sum by (le, function, module) (rate({bucket_name}{function_label}[5m]))");
    let latency = format!(
        "# 95th and 99th percentile latencies
histogram_quantile(0.99, {latency}) or
histogram_quantile(0.95, {latency})"
    );
    let latency_doc = format!(
        "- [Latency (95th and 99th percentiles)]({})",
        make_prometheus_url(&prometheus_url, &latency)
    );

    // Concurrent calls
    let concurrent_calls = format!(
        "# Concurrent calls to the `{function_name}` function

sum by (function, module) {gauge_name}{function_label}"
    );
    let concurrent_calls_doc = format!(
        "- [Concurrent calls]({})",
        make_prometheus_url(&prometheus_url, &concurrent_calls)
    );

    // Create the RustDoc string
    format!(
        "\n\n---

## Autometrics

View the live metrics for this function:
{request_rate_doc}
{error_rate_doc}
{latency_doc}
{concurrent_calls_doc}

This function has the following metrics associated with it:
- `{counter_name}{function_label}`
- `{bucket_name}{function_label}`
- `{gauge_name}{function_label}`"
    )
}

fn make_prometheus_url(url: &str, query: &str) -> String {
    let mut url = url.to_string();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("graph?g0.expr=");
    url.push_str(&urlencoding::encode(query));
    // Go straight to the graph tab
    url.push_str("&g0.tab=0");
    url
}
