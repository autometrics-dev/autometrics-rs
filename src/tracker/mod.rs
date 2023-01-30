use crate::labels::Label;

#[cfg(feature = "opentelemetry")]
mod opentelemetry;
#[cfg(feature = "prometheus")]
mod prometheus;

#[cfg(all(
    feature = "metrics",
    not(any(feature = "opentelemetry", feature = "prometheus"))
))]
pub use self::metrics::MetricsTracker as AutometricsTracker;
#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetryTracker as AutometricsTracker;
#[cfg(all(feature = "prometheus", not(feature = "opentelemetry")))]
pub use self::prometheus::PrometheusTracker as AutometricsTracker;

pub trait TrackMetrics {
    fn function(&self) -> &'static str;
    fn module(&self) -> &'static str;
    fn start(function: &'static str, module: &'static str) -> Self;
    fn finish<'a>(self, counter_labels: &[Label]);
}
