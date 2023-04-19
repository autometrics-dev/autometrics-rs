use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};

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
    fn set_build_info(build_info_labels: &BuildInfoLabels);
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self;
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels);
}
