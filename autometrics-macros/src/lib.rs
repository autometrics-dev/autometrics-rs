use crate::parse::{AutometricsArgs, Item};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use proc_macro2::TokenStream;
use quote::quote;
use std::env;
use syn::{parse_macro_input, ImplItem, ItemFn, ItemImpl, Result};

mod parse;
mod result_labels;

const COUNTER_NAME_PROMETHEUS: &str = "function_calls_count";
const HISTOGRAM_BUCKET_NAME_PROMETHEUS: &str = "function_calls_duration_bucket";
const GAUGE_NAME_PROMETHEUS: &str = "function_calls_concurrent";
const ADD_BUILD_INFO_LABELS: &str =
    "* on (instance, job) group_left(version, commit) last_over_time(build_info[1s])";

const DEFAULT_PROMETHEUS_URL: &str = "http://localhost:9090";

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

#[proc_macro_derive(ResultLabels, attributes(label))]
pub fn result_labels(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    result_labels::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
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
                use autometrics::__private::{CALLER, CounterLabels, GetStaticStrFromIntoStaticStr, GetStaticStr};
                let result_label = #result_label;
                // If the return type implements Into<&'static str>, attach that as a label
                let value_type = (&result).__autometrics_static_str();
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
        quote! {
            {
                use autometrics::__private::{CALLER, CounterLabels, GetLabels};
                let result_labels = autometrics::result_labels!(&result);
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
                use autometrics::__private::{AutometricsTracker, BuildInfoLabels, TrackMetrics};
                AutometricsTracker::set_build_info(&BuildInfoLabels::new(
                    option_env!("AUTOMETRICS_VERSION").or(option_env!("CARGO_PKG_VERSION")).unwrap_or_default(),
                    option_env!("AUTOMETRICS_COMMIT").or(option_env!("VERGEN_GIT_SHA")).unwrap_or_default(),
                    option_env!("AUTOMETRICS_BRANCH").or(option_env!("VERGEN_GIT_BRANCH")).unwrap_or_default(),
                ));
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
    format!("sum by (function, module, commit, version) (rate({counter_name}{{{label_key}=\"{label_value}\"}}[5m]) {ADD_BUILD_INFO_LABELS})")
}

fn error_ratio_query(counter_name: &str, label_key: &str, label_value: &str) -> String {
    let request_rate = request_rate_query(counter_name, label_key, label_value);
    format!("(sum by (function, module, commit, version) (rate({counter_name}{{{label_key}=\"{label_value}\",result=\"error\"}}[5m]) {ADD_BUILD_INFO_LABELS}))
/
({request_rate})",)
}

fn latency_query(bucket_name: &str, label_key: &str, label_value: &str) -> String {
    let latency = format!(
        "sum by (le, function, module, commit, version) (rate({bucket_name}{{{label_key}=\"{label_value}\"}}[5m]) {ADD_BUILD_INFO_LABELS})"
    );
    format!(
        "histogram_quantile(0.99, {latency}) or
histogram_quantile(0.95, {latency})"
    )
}

fn concurrent_calls_query(gauge_name: &str, label_key: &str, label_value: &str) -> String {
    format!("sum by (function, module, commit, version) ({gauge_name}{{{label_key}=\"{label_value}\"}} {ADD_BUILD_INFO_LABELS})")
}
