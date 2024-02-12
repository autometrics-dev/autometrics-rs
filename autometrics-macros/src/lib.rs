use crate::parse::{AutometricsArgs, Item};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use regex::Regex;
use std::env;
use std::str::FromStr;
use syn::{
    parse_macro_input, GenericArgument, ImplItem, ItemFn, ItemImpl, PathArguments, Result,
    ReturnType, Type,
};

mod parse;
mod result_labels;

const ADD_BUILD_INFO_LABELS: &str =
    "* on (instance, job) group_left(version, commit) last_over_time(build_info[1s])";

const DEFAULT_PROMETHEUS_URL: &str = "http://localhost:9090";

#[proc_macro_attribute]
pub fn autometrics(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AutometricsArgs);

    let async_trait = check_async_trait(&item);
    let item = parse_macro_input!(item as Item);

    let result = match item {
        Item::Function(item) => instrument_function(&args, item, args.struct_name.as_deref()),
        Item::Impl(item) => instrument_impl_block(&args, item, &async_trait),
    };

    let output = match result {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

/// returns the `async_trait` attributes that have to be re-added after our instrumentation magic has been added
fn check_async_trait(input: &proc_macro::TokenStream) -> String {
    let regex = Regex::new(r#"#\[[^\]]*async_trait\]"#)
        .expect("The regex is hardcoded and thus guaranteed to be successfully parseable");

    let original = input.to_string();
    let attributes: Vec<_> = regex.find_iter(&original).map(|m| m.as_str()).collect();

    attributes.join("\n")
}

#[proc_macro_derive(ResultLabels, attributes(label))]
pub fn result_labels(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    result_labels::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Add autometrics instrumentation to a single function
fn instrument_function(
    args: &AutometricsArgs,
    item: ItemFn,
    struct_name: Option<&str>,
) -> Result<TokenStream> {
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;
    let attrs = item.attrs;

    // Methods are identified as Struct::method
    let function_name = match struct_name {
        Some(struct_name) => format!("{}::{}", struct_name, sig.ident),
        None => sig.ident.to_string(),
    };

    // The PROMETHEUS_URL can be configured by passing the environment variable during build time
    let prometheus_url =
        env::var("PROMETHEUS_URL").unwrap_or_else(|_| DEFAULT_PROMETHEUS_URL.to_string());

    // Build the documentation we'll add to the function's RustDocs, unless it is disabled by the environment variable
    let metrics_docs = if env::var("AUTOMETRICS_DISABLE_DOCS").is_ok() {
        String::new()
    } else {
        create_metrics_docs(&prometheus_url, &function_name, args.track_concurrency)
    };

    // Type annotation to allow type inference to work on return expressions (such as `.collect()`), as
    // well as prevent compiler type-inference from selecting the wrong branch in the `spez` macro later.
    //
    // Type inference can make the compiler select one of the early cases of `autometrics::result_labels!`
    // even if the types `T` or `E` do not implement the `GetLabels` trait. That leads to a compilation error
    // looking like this:
    // ```
    // error[E0277]: the trait bound `ApiError: GetLabels` is not satisfied
    //  --> examples/full-api/src/routes.rs:48:1
    //   |
    //48 | #[autometrics(objective = API_SLO)]
    //   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `GetLabels` is not implemented for `ApiError`
    //   |
    //   = help: the trait `create_user::{closure#0}::Match2` is implemented for `&&&&create_user::{closure#0}::Match<&Result<T, E>>`
    //note: required for `&&&&create_user::{closure#0}::Match<&Result<Json<User>, ApiError>>` to implement `create_user::{closure#0}::Match2`
    //  --> examples/full-api/src/routes.rs:48:1
    //   |
    //48 | #[autometrics(objective = API_SLO)]
    //   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //   = note: this error originates in the macro `$crate::__private::spez` which comes from the expansion of the attribute macro `autometrics` (in Nightly builds, run with -Z macro-backtrace for more info)
    // ```
    //
    // specifying the return type makes the compiler select the (correct) fallback case of `ApiError` not being a
    // `GetLabels` implementor.
    let return_type = match sig.output {
        ReturnType::Default => quote! { : () },
        ReturnType::Type(_, ref t) => match t.as_ref() {
            Type::ImplTrait(_) => quote! {},
            Type::Path(path) => {
                let mut ts = vec![];
                let mut first = true;

                for segment in &path.path.segments {
                    let ident = &segment.ident;
                    let args = &segment.arguments;

                    // special handling in case the type is angle bracket with a `impl` trait
                    // in such a case, we would run into the following error
                    //
                    // ```
                    // error[E0562]: `impl Trait` only allowed in function and inherent method return types, not in variable bindings
                    //   --> src/main.rs:11:28
                    //    |
                    // 11 | async fn hello() -> Result<impl ToString, std::io::Error> {
                    //    |                            ^^^^^^^^^^^^^
                    // ```
                    //
                    // this whole block just re-creates the angle bracketed `<impl ToString, std::io::Error>`
                    // manually but the trait `impl` replaced with an infer `_`, which fixes this issue
                    let suffix = match args {
                        PathArguments::AngleBracketed(brackets) => {
                            let mut ts = vec![];

                            for args in &brackets.args {
                                ts.push(match args {
                                    GenericArgument::Type(Type::ImplTrait(_)) => {
                                        quote! { _ }
                                    }
                                    generic_arg => quote! { #generic_arg },
                                });
                            }

                            quote! { ::<#(#ts),*> }
                        }
                        _ => quote! {},
                    };

                    // primitive way to check whenever this is the first iteration or not
                    // as on the first iteration, we don't want to prepend `::`,
                    // as types may be local and/or imported and then couldn't be found
                    if !first {
                        ts.push(quote! { :: });
                    } else {
                        first = false;
                    }

                    ts.push(quote! { #ident });
                    ts.push(quote! { #suffix });
                }

                quote! { : #(#ts)* }
            }
            _ => quote! { : #t },
        },
    };

    // Track the name and module of the current function as a task-local variable
    // so that any functions it calls know which function they were called by
    let caller_info = quote! {
        use autometrics::__private::{CALLER, CallerInfo};
        let caller = CallerInfo {
            caller_function: #function_name,
            caller_module: module_path!(),
        };
    };

    // Wrap the body of the original function, using a slightly different approach based on whether the function is async
    let call_function = if sig.asyncness.is_some() {
        quote! {
            {
                #caller_info
                CALLER.scope(caller, async move {
                    #block
                }).await
            }
        }
    } else {
        quote! {
            {
                #caller_info
                CALLER.sync_scope(caller, move || {
                    #block
                })
            }
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
                let caller = CALLER.get();
                CounterLabels::new(
                    #function_name,
                    module_path!(),
                    caller.caller_function,
                    caller.caller_module,
                    Some((result_label, value_type)),
                    #objective,
                )
            }
        }
    } else {
        quote! {
            {
                use autometrics::__private::{CALLER, CounterLabels, GetLabels};
                let result_labels = autometrics::get_result_labels_for_value!(&result);
                let caller = CALLER.get();
                CounterLabels::new(
                    #function_name,
                    module_path!(),
                    caller.caller_function,
                    caller.caller_module,
                    result_labels,
                    #objective,
                )
            }
        }
    };

    let gauge_labels = if args.track_concurrency {
        quote! { {
            use autometrics::__private::GaugeLabels;
            Some(&GaugeLabels::new(
                #function_name,
                module_path!(),
            )) }
        }
    } else {
        quote! { None }
    };

    // This is a little nuts.
    // In debug mode, we're using the `linkme` crate to collect all the function descriptions into a static slice.
    // We're then using that to start all the function counters at zero, even before the function is called.
    let collect_function_descriptions = if cfg!(debug_assertions) {
        quote! {
            {
                use autometrics::__private::{linkme::distributed_slice, FUNCTION_DESCRIPTIONS, FunctionDescription};
                #[distributed_slice(FUNCTION_DESCRIPTIONS)]
                // Point the distributed_slice macro to the linkme crate re-exported from autometrics
                #[linkme(crate = autometrics::__private::linkme)]
                static FUNCTION_DESCRIPTION: FunctionDescription = FunctionDescription {
                    name: #function_name,
                    module: module_path!(),
                    objective: #objective,
                };
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #(#attrs)*

        // Append the metrics documentation to the end of the function's documentation
        #[doc=#metrics_docs]

        #vis #sig {
            #collect_function_descriptions

            let __autometrics_tracker = {
                use autometrics::__private::{AutometricsTracker, BuildInfoLabels, TrackMetrics};
                AutometricsTracker::set_build_info(&BuildInfoLabels::new(
                    option_env!("AUTOMETRICS_VERSION").or(option_env!("CARGO_PKG_VERSION")).unwrap_or_default(),
                    option_env!("AUTOMETRICS_COMMIT").or(option_env!("VERGEN_GIT_SHA")).unwrap_or_default(),
                    option_env!("AUTOMETRICS_BRANCH").or(option_env!("VERGEN_GIT_BRANCH")).unwrap_or_default(),
                ));
                AutometricsTracker::start(#gauge_labels)
            };

            let result #return_type = #call_function;

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
fn instrument_impl_block(
    args: &AutometricsArgs,
    mut item: ItemImpl,
    attributes_to_re_add: &str,
) -> Result<TokenStream> {
    let struct_name = Some(item.self_ty.to_token_stream().to_string());

    // Replace all of the method items in place
    item.items = item
        .items
        .into_iter()
        .map(|item| match item {
            ImplItem::Fn(mut method) => {
                // Skip any methods that have the #[skip_autometrics] attribute
                if method
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("skip_autometrics"))
                {
                    method
                        .attrs
                        .retain(|attr| !attr.path().is_ident("skip_autometrics"));
                    return ImplItem::Fn(method);
                }

                let item_fn = ItemFn {
                    attrs: method.attrs,
                    vis: method.vis,
                    sig: method.sig,
                    block: Box::new(method.block),
                };
                let tokens = match instrument_function(args, item_fn, struct_name.as_deref()) {
                    Ok(tokens) => tokens,
                    Err(err) => err.to_compile_error(),
                };
                ImplItem::Verbatim(tokens)
            }
            _ => item,
        })
        .collect();

    let ts = TokenStream::from_str(attributes_to_re_add)?;

    Ok(quote! {
        #ts
        #item
    })
}

/// Create Prometheus queries for the generated metric and
/// package them up into a RustDoc string
fn create_metrics_docs(prometheus_url: &str, function: &str, track_concurrency: bool) -> String {
    let request_rate = request_rate_query("function", function);
    let request_rate_url = make_prometheus_url(
        prometheus_url,
        &request_rate,
        &format!(
            "Rate of calls to the `{function}` function per second, averaged over 5 minute windows"
        ),
    );
    let callee_request_rate = request_rate_query("caller_function", function);
    let callee_request_rate_url = make_prometheus_url(prometheus_url, &callee_request_rate, &format!("Rate of calls to functions called by `{function}` per second, averaged over 5 minute windows"));

    let error_ratio = &error_ratio_query("function", function);
    let error_ratio_url = make_prometheus_url(prometheus_url, error_ratio, &format!("Percentage of calls to the `{function}` function that return errors, averaged over 5 minute windows"));
    let callee_error_ratio = &error_ratio_query("caller_function", function);
    let callee_error_ratio_url = make_prometheus_url(prometheus_url, callee_error_ratio, &format!("Percentage of calls to functions called by `{function}` that return errors, averaged over 5 minute windows"));

    let latency = latency_query("function", function);
    let latency_url = make_prometheus_url(
        prometheus_url,
        &latency,
        &format!("95th and 99th percentile latencies (in seconds) for the `{function}` function"),
    );

    // Only include the concurrent calls query if the user has enabled it for this function
    let concurrent_calls_doc = if track_concurrency {
        let concurrent_calls = concurrent_calls_query("function", function);
        let concurrent_calls_url = make_prometheus_url(
            prometheus_url,
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

fn request_rate_query(label_key: &str, label_value: &str) -> String {
    format!("sum by (function, module, service_name, commit, version) (rate({{__name__=~\"function_calls(_count)?(_total)?\",{label_key}=\"{label_value}\"}}[5m]) {ADD_BUILD_INFO_LABELS})")
}

fn error_ratio_query(label_key: &str, label_value: &str) -> String {
    let request_rate = request_rate_query(label_key, label_value);
    format!("(sum by (function, module, service_name, commit, version) (rate({{__name__=~\"function_calls(_count)?(_total)?\",{label_key}=\"{label_value}\",result=\"error\"}}[5m]) {ADD_BUILD_INFO_LABELS}))
/
({request_rate})",)
}

fn latency_query(label_key: &str, label_value: &str) -> String {
    let latency = format!(
        "sum by (le, function, module, service_name, commit, version) (rate({{__name__=~\"function_calls_duration(_seconds)?_bucket\",{label_key}=\"{label_value}\"}}[5m]) {ADD_BUILD_INFO_LABELS})"
    );
    format!(
        "label_replace(histogram_quantile(0.99, {latency}), \"percentile_latency\", \"99\", \"\", \"\")
or
label_replace(histogram_quantile(0.95, {latency}), \"percentile_latency\", \"95\", \"\", \"\")"
    )
}

fn concurrent_calls_query(label_key: &str, label_value: &str) -> String {
    format!("sum by (function, module, service_name, commit, version) (function_calls_concurrent{{{label_key}=\"{label_value}\"}} {ADD_BUILD_INFO_LABELS})")
}
