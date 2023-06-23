// Use the unstable `doc_cfg` feature when docs.rs is building the documentation
// https://stackoverflow.com/questions/61417452/how-to-get-a-feature-requirement-tag-in-the-documentation-generated-by-cargo-do/61417700#61417700
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]
#![doc = include_str!("README.md")]

pub mod backends;
mod constants;
#[cfg(any(
    feature = "exemplars-tracing",
    feature = "exemplars-tracing-opentelemetry"
))]
pub mod exemplars;
mod labels;
pub mod objectives;
#[cfg(feature = "prometheus-exporter")]
pub mod prometheus_exporter;
mod task_local;
mod tracker;

/// A macro that makes it easy to instrument functions with the most useful metrics.
///
/// ## Example
/// ```
/// use autometrics::autometrics;
///
/// #[autometrics]
/// pub async fn create_user() {
///     // Now this function has metrics!
/// }
///
/// struct MyStruct;
///
/// #[autometrics]
/// impl MyStruct {
///     #[skip_autometrics]
///     pub fn new() -> Self {
///        Self
///     }
///
///     fn my_method(&self) {
///        // This method has metrics too!
///    }
/// }
/// ```
///
/// ## Optional Parameters
///
/// ### `ok_if` and `error_if`
///
/// Example:
/// ```rust
/// # use autometrics::autometrics;
/// #[autometrics(ok_if = Option::is_some)]
/// pub fn db_load_key(key: &str) -> Option<String> {
///   None
/// }
/// ```
///
/// If the function does not return a `Result`, you can use `ok_if` and `error_if` to specify
/// whether the function call was "successful" or not, as far as the metrics are concerned.
///
/// For example, if a function returns an HTTP response, you can use `ok_if` or `error_if` to
/// add the `result` label based on the status code:
/// ```rust
/// # use autometrics::autometrics;
/// # use http::{Request, Response};
///
/// fn is_success<T>(res: &Response<T>) -> bool {
///     res.status().is_success()
/// }
///
/// #[autometrics(ok_if = is_success)]
/// pub async fn my_handler(req: Request<()>) -> Response<()> {
/// # Response::new(())
///     // ...
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
/// # use autometrics::autometrics;
/// #[autometrics(track_concurrency)]
/// pub fn queue_task() { }
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
///     .success_rate(ObjectivePercentile::P99_9);
///
/// #[autometrics(objective = API_SLO)]
/// pub fn handler() {
///    // ...
/// }
/// ```
///
/// Include this function's metrics in the specified [`Objective`].
///
/// [`Objective`]: crate::objectives::Objective
pub use autometrics_macros::autometrics;

/// # Customize how types map to the Autometrics `result` label.
///
/// The `ResultLabels` derive macro allows you to specify
/// whether the variants of an enum should be considered as errors or
/// successes from the perspective of the generated metrics.
///
/// For example, this would allow you to ignore the client-side HTTP errors (400-499)
/// from the function's success rate and any Service-Level Objectives (SLOs) it is part of.
///
/// Putting such a policy in place would look like this in code:
///
/// ```rust,ignore
/// use autometrics::ResultLabels
///
/// #[derive(ResultLabels)]
/// pub enum ServiceError {
///     // By default, the variant will be inferred by the context,
///     // so you do not need to decorate every variant.
///     // - if ServiceError::Database is in an `Err(_)` variant, it will be an "error",
///     // - if ServiceError::Database is in an `Ok(_)` variant, it will be an "ok",
///     // - otherwise, no label will be added
///     Database,
///     // It is possible to mention it as well of course.
///     // Only "error" and "ok" are accepted values
///     //
///     // Forcing "error" here means that even returning `Ok(ServiceError::Network)`
///     // from a function will count as an error for autometrics.
///     #[label(result = "error")]
///     Network,
///     // Forcing "ok" here means that even returning `Err(ServiceError::Authentication)`
///     // from a function will count as a success for autometrics.
///     #[label(result = "ok")]
///     Authentication,
///     #[label(result = "ok")]
///     Authorization,
/// }
///
/// pub type ServiceResult<T> = Result<T, ServiceError>;
/// ```
///
/// With these types, whenever a function returns a `ServiceResult`, having a
/// `ServiceError::Authentication` or `Authorization` would _not_ count as a
/// failure from your handler that should trigger alerts and consume the "error
/// budget" of the service.
///
/// ## Per-function labelling
///
/// The `ResultLabels` macro does _not_ have the granularity to behave
/// differently on different functions: if returning
/// `ServiceError::Authentication` from `function_a` is "ok", then returning
/// `ServiceError::Authentication` from `function_b` will be "ok" too.
///
/// To work around this, you must use the `ok_if` or `error_if` arguments to the
/// [autometrics](crate::autometrics) invocation on `function_b`: those
/// directives have priority over the ResultLabels annotations.
pub use autometrics_macros::ResultLabels;

// Optional exports
#[cfg(feature = "prometheus-exporter")]
#[deprecated(
    since = "0.5.0",
    note = "Use autometrics::prometheus_exporter::encode_to_string instead. This will be removed in v0.6"
)]
/// Replaced by [`prometheus_exporter::encode_to_string`].
pub fn encode_global_metrics() -> Result<String, prometheus_exporter::EncodingError> {
    prometheus_exporter::encode_to_string()
}
#[cfg(feature = "prometheus-exporter")]
#[deprecated(
    since = "0.5.0",
    note = "Use autometrics::prometheus_exporter::init instead. This will be removed in v0.6"
)]
/// Replaced by [`prometheus_exporter::init`].
pub fn global_metrics_exporter() -> prometheus_exporter::GlobalPrometheus {
    prometheus_exporter::init();
    prometheus_exporter::GLOBAL_EXPORTER.clone()
}

/// We use the histogram buckets recommended by the OpenTelemetry specification
/// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#explicit-bucket-histogram-aggregation
#[cfg(any(prometheus, prometheus_client, prometheus_exporter))]
pub(crate) const HISTOGRAM_BUCKETS: [f64; 14] = [
    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

/// Non-public API, used by the autometrics macro.
// Note that this needs to be publicly exported (despite being called private)
// because it is used by code generated by the autometrics macro.
// We could move more or all of the code into the macro itself.
// However, the compiler would need to compile a lot of duplicate code in every
// instrumented function. It's also harder to develop and maintain macros with
// too much generated code, because rust-analyzer treats the macro code as a kind of string
// so you don't get any autocompletion or type checking.
#[doc(hidden)]
pub mod __private {
    use crate::task_local::LocalKey;
    use once_cell::sync::OnceCell;
    use std::{cell::RefCell, thread_local};

    pub use crate::constants::*;
    pub use crate::labels::*;
    pub use crate::tracker::{AutometricsTracker, TrackMetrics};
    pub use spez::spez;

    /// Task-local value used for tracking which function called the current function
    pub static CALLER: LocalKey<&'static str> = {
        // This does the same thing as the tokio::thread_local macro with the exception that
        // it initializes the value with the empty string.
        // The tokio macro does not allow you to get the value before setting it.
        // However, in our case, we want it to simply return the empty string rather than panicking.
        thread_local! {
            static CALLER_KEY: RefCell<Option<&'static str>> = const { RefCell::new(Some("")) };
        }

        LocalKey { inner: CALLER_KEY }
    };

    /// Load the service name from one of the following runtime environment variables:
    /// - `AUTOMETRICS_SERVICE_NAME`
    /// - `OTEL_SERVICE_NAME`
    /// Or, fall back to the name of the cargo package defined in the `Cargo.toml` file.
    pub fn service_name(cargo_pkg_name: &'static str) -> &'static str {
        // Note that we are using a OnceCell here because we want the label value to
        // be available as a &'static str. This allows us to load the value from the
        // environment variables once at startup while still accessing it as a &'static str
        static SERVICE_NAME: OnceCell<String> = OnceCell::new();
        SERVICE_NAME.get_or_init(|| {
            std::env::var("AUTOMETRICS_SERVICE_NAME")
                .or_else(|_| std::env::var("OTEL_SERVICE_NAME"))
                .unwrap_or_else(|| cargo_pkg_name.to_string())
        })
    }
}
