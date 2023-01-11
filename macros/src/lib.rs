use crate::parse::Args;
use proc_macro2::TokenStream;
use quote::quote;
use std::env;
use syn::{parse_macro_input, ItemFn, Result};

mod parse;

const DEFAULT_METRIC_BASE_NAME: &str = "function";
const DEFAULT_PROMETHEUS_URL: &str = "http://localhost:9090";

/// Autometrics instruments your functions with automatically generated metrics
/// and writes Prometheus queries for you, making it easy for you to observe and
/// understand how your system performs in production.
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

    let track_metrics = quote! {
        use autometrics::__private::{Context, create_labels, create_labels_with_result, GetLabels, GetLabelsFromResult, register_histogram, str_replace};

        // Metric labels must use underscores as separators rather than colons.
        // The str_replace macro produces a static str rather than a String.
        // Note that we cannot determine the module path at macro expansion time
        // (see https://github.com/rust-lang/rust/issues/54725), only at compile/run time
        let module_path = str_replace!(module_path!(), "::", "_");

        let histogram = register_histogram(#histogram_name);
        let context = Context::current();

        // Note that the Rust compiler should optimize away this if/else statement because
        // it's smart enough to figure out that only one branch will ever be hit for a given function
        if let Some(result) = ret.__metrics_attributes_get_result_label() {
            histogram.record(&context, duration, &create_labels_with_result(#function_name, module_path, result));
        } else {
            histogram.record(&context, duration, &create_labels(#function_name, module_path));
        }
    };

    let metrics_docs = create_metrics_docs(&prometheus_url, &histogram_name, &function_name);

    Ok(quote! {
        #(#attrs)*

        // Append the metrics documentation to the end of the function's documentation
        #[doc=#metrics_docs]

        #vis #sig {
            let __autometrics_start = ::std::time::Instant::now();

            let ret = #maybe_async { #block } #maybe_await;

            {
                let duration = __autometrics_start.elapsed().as_secs_f64();
                #track_metrics
            }

            ret
        }
    })
}

/// Create Prometheus queries for the generated metric and
/// package them up into a RustDoc string
fn create_metrics_docs(prometheus_url: &str, histogram_name: &str, function_name: &str) -> String {
    let counter_name = format!("{histogram_name}_count");
    let bucket_name = format!("{histogram_name}_bucket");
    let function_label = format!("{{function=\"{function_name}\"}}");

    // Request rate
    let request_rate =
        format!("sum by (function, module) (rate({counter_name}{function_label}[5m]))");
    let request_rate_doc = format!("# Rate of calls to the `{function_name}` function per second, averaged over 5 minute windows\n{request_rate}");
    let request_rate_doc = format!(
        "- [Request Rate]({})",
        make_prometheus_url(&prometheus_url, &request_rate_doc)
    );

    // Error rate
    let error_rate = format!("# Percentage of calls to the `{function_name}` function that return errors, averaged over 5 minute windows
sum by (function, module) (rate({counter_name}{{function=\"{function_name}\",result=\"err\"}}[5m])) / {request_rate}");
    let error_rate_doc = format!(
        "\n- [Error Rate]({})",
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

    // Create the RustDoc string
    format!(
        "\n\n## Metrics

View the live metrics for this function:
{request_rate_doc}{error_rate_doc}
{latency_doc}

This function has the following metrics associated with it:
- `{counter_name}{{function=\"{function_name}\"}}`
- `{bucket_name}{{function=\"{function_name}\"}}`",
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
