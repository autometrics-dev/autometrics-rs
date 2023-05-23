use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};

#[cfg(feature = "metrics")]
mod metrics;
#[cfg(feature = "opentelemetry")]
mod opentelemetry;
#[cfg(feature = "prometheus")]
mod prometheus;
#[cfg(feature = "prometheus-client")]
pub(crate) mod prometheus_client;

#[cfg(feature = "metrics")]
pub use self::metrics::MetricsTracker;
#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetryTracker;
#[cfg(feature = "prometheus")]
pub use self::prometheus::PrometheusTracker;
#[cfg(feature = "prometheus-client")]
pub use self::prometheus_client::PrometheusClientTracker;

#[cfg(any(
    all(
        feature = "metrics",
        any(
            feature = "opentelemetry",
            feature = "prometheus",
            feature = "prometheus-client"
        )
    ),
    all(
        feature = "opentelemetry",
        any(feature = "prometheus", feature = "prometheus-client")
    ),
    all(feature = "prometheus", feature = "prometheus-client")
))]
compile_error!("Only one of the metrics, opentelemetry, prometheus, or prometheus-client features can be enabled at a time");

pub trait TrackMetrics {
    fn set_build_info(build_info_labels: &BuildInfoLabels);
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self;
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels);
}

pub struct AutometricsTracker {
    #[cfg(feature = "metrics")]
    metrics_tracker: MetricsTracker,
    #[cfg(feature = "opentelemetry")]
    opentelemetry_tracker: OpenTelemetryTracker,
    #[cfg(feature = "prometheus")]
    prometheus_tracker: PrometheusTracker,
    #[cfg(feature = "prometheus-client")]
    prometheus_client_tracker: PrometheusClientTracker,
}

impl TrackMetrics for AutometricsTracker {
    #[allow(unused_variables)]
    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        #[cfg(feature = "metrics")]
        MetricsTracker::set_build_info(build_info_labels);
        #[cfg(feature = "opentelemetry")]
        OpenTelemetryTracker::set_build_info(build_info_labels);
        #[cfg(feature = "prometheus")]
        PrometheusTracker::set_build_info(build_info_labels);
        #[cfg(feature = "prometheus-client")]
        PrometheusClientTracker::set_build_info(build_info_labels);
    }

    #[allow(unused_variables)]
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        Self {
            #[cfg(feature = "metrics")]
            metrics_tracker: MetricsTracker::start(gauge_labels),
            #[cfg(feature = "opentelemetry")]
            opentelemetry_tracker: OpenTelemetryTracker::start(gauge_labels),
            #[cfg(feature = "prometheus")]
            prometheus_tracker: PrometheusTracker::start(gauge_labels),
            #[cfg(feature = "prometheus-client")]
            prometheus_client_tracker: PrometheusClientTracker::start(gauge_labels),
        }
    }

    #[allow(unused_variables)]
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        #[cfg(feature = "metrics")]
        self.metrics_tracker
            .finish(counter_labels, histogram_labels);
        #[cfg(feature = "opentelemetry")]
        self.opentelemetry_tracker
            .finish(counter_labels, histogram_labels);
        #[cfg(feature = "prometheus")]
        self.prometheus_tracker
            .finish(counter_labels, histogram_labels);
        #[cfg(feature = "prometheus-client")]
        self.prometheus_client_tracker
            .finish(counter_labels, histogram_labels);
    }
}
