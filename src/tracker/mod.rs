use crate::labels::Label;

#[cfg(feature = "opentelemetry")]
mod opentelemetry;
// #[cfg(feature = "prometheus")]
// mod prometheus;

#[cfg(all(
    feature = "metrics",
    not(any(feature = "opentelemetry", feature = "prometheus"))
))]
pub use self::metrics::MetricsTracker as AutometricsTracker;
#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetryTracker as AutometricsTracker;
// #[cfg(all(feature = "prometheus", not(feature = "opentelemetry")))]
// pub use self::prometheus::PrometheusTracker as AutometricsTracker;

pub(crate) const COUNTER_NAME: &str = "function.calls.count";
pub(crate) const COUNTER_DESCRIPTION: &str = "Autometrics counter for tracking function calls";
pub(crate) const HISTOGRAM_NAME: &str = "function.calls.duration";
pub(crate) const HISTOGRAM_DESCRIPTION: &str =
    "Autometrics histogram for tracking function call duration";
pub(crate) const GAUGE_NAME: &str = "function.calls.concurrent";
pub(crate) const GAUGE_DESCRIPTION: &str =
    "Autometrics gauge for tracking concurrent function calls";

pub trait TrackMetrics {
    fn function(&self) -> &'static str;
    fn module(&self) -> &'static str;
    fn start(function: &'static str, module: &'static str) -> Self;
    fn finish<'a>(self, counter_labels: &[Label]);
}
