use crate::labels::Label;

#[cfg(feature = "metrics")]
mod metrics;
#[cfg(feature = "opentelemetry")]
mod opentelemetry;
#[cfg(feature = "prometheus")]
mod prometheus;

// By default, use the opentelemetry crate
#[cfg(all(
    feature = "opentelemetry",
    not(any(feature = "metrics", feature = "prometheus"))
))]
pub use self::opentelemetry::OpenTelemetryTracker as AutometricsTracker;

// But use one of the other crates if either of those features are enabled
#[cfg(all(feature = "metrics", not(feature = "prometheus")))]
pub use self::metrics::MetricsTracker as AutometricsTracker;
#[cfg(feature = "prometheus")]
pub use self::prometheus::PrometheusTracker as AutometricsTracker;

pub trait TrackMetrics {
    fn function(&self) -> &'static str;
    fn module(&self) -> &'static str;
    fn start(function: &'static str, module: &'static str, track_concurrency: bool) -> Self;
    fn finish<'a>(self, counter_labels: &[Label]);
}
