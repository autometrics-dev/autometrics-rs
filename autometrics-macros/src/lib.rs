use crate::parse::Item;
use syn::parse_macro_input;

mod instrument;
mod objectives;
mod parse;

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
        Item::Function(item) => instrument::instrument_function(&args, item),
        Item::Impl(item) => instrument::instrument_impl_block(&args, item),
    };

    let output = match result {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

#[proc_macro]
pub fn create_objective(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as parse::CreateObjectiveArgs);
    objectives::create_objective(input).into()
}
