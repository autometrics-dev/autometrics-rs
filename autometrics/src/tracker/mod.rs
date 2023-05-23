use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};

#[cfg(feature = "metrics")]
mod metrics;
#[cfg(feature = "opentelemetry")]
mod opentelemetry;
#[cfg(feature = "prometheus")]
mod prometheus;
#[cfg(feature = "prometheus-client")]
pub(crate) mod prometheus_client;

// Priority if multiple features are enabled:
// 1. prometheus
// 2. prometheus-client
// 3. metrics
// 4. opentelemetry (default)

// By default, use the opentelemetry crate
#[cfg(all(
    feature = "opentelemetry",
    not(any(
        feature = "metrics",
        feature = "prometheus",
        feature = "prometheus-client"
    ))
))]
pub use self::opentelemetry::OpenTelemetryTracker as AutometricsTracker;

// But use one of the other crates if any of those features are enabled
#[cfg(all(
    feature = "metrics",
    not(any(feature = "prometheus", feature = "prometheus-client"))
))]
pub use self::metrics::MetricsTracker as AutometricsTracker;
#[cfg(feature = "prometheus")]
pub use self::prometheus::PrometheusTracker as AutometricsTracker;
#[cfg(all(feature = "prometheus-client", not(feature = "prometheus")))]
pub use self::prometheus_client::PrometheusClientTracker as AutometricsTracker;

pub trait TrackMetrics {
    fn set_build_info(build_info_labels: &BuildInfoLabels);
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self;
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels);
}
