#[cfg(debug_assertions)]
use crate::__private::FunctionDescription;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};

#[cfg(metrics)]
mod metrics;
#[cfg(opentelemetry)]
mod opentelemetry;
#[cfg(prometheus)]
mod prometheus;
#[cfg(prometheus_client)]
pub(crate) mod prometheus_client;

#[cfg(metrics)]
pub use self::metrics::MetricsTracker;
#[cfg(opentelemetry)]
pub use self::opentelemetry::OpenTelemetryTracker;
#[cfg(prometheus)]
pub use self::prometheus::PrometheusTracker;
#[cfg(prometheus_client)]
pub use self::prometheus_client::PrometheusClientTracker;

#[cfg(all(
    not(doc),
    any(
        all(metrics, any(opentelemetry, prometheus, prometheus_client)),
        all(opentelemetry, any(prometheus, prometheus_client)),
        all(prometheus, prometheus_client)
    )
))]
compile_error!("Only one of the metrics, opentelemetry, prometheus, or prometheus-client features can be enabled at a time");

pub trait TrackMetrics {
    fn set_build_info(build_info_labels: &BuildInfoLabels);
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self;
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels);
    #[cfg(debug_assertions)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]);
}

pub struct AutometricsTracker {
    #[cfg(metrics)]
    metrics_tracker: MetricsTracker,
    #[cfg(opentelemetry)]
    opentelemetry_tracker: OpenTelemetryTracker,
    #[cfg(prometheus)]
    prometheus_tracker: PrometheusTracker,
    #[cfg(prometheus_client)]
    prometheus_client_tracker: PrometheusClientTracker,
}

impl TrackMetrics for AutometricsTracker {
    #[allow(unused_variables)]
    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        #[cfg(metrics)]
        MetricsTracker::set_build_info(build_info_labels);
        #[cfg(opentelemetry)]
        OpenTelemetryTracker::set_build_info(build_info_labels);
        #[cfg(prometheus)]
        PrometheusTracker::set_build_info(build_info_labels);
        #[cfg(prometheus_client)]
        PrometheusClientTracker::set_build_info(build_info_labels);
    }

    #[allow(unused_variables)]
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        Self {
            #[cfg(metrics)]
            metrics_tracker: MetricsTracker::start(gauge_labels),
            #[cfg(opentelemetry)]
            opentelemetry_tracker: OpenTelemetryTracker::start(gauge_labels),
            #[cfg(prometheus)]
            prometheus_tracker: PrometheusTracker::start(gauge_labels),
            #[cfg(prometheus_client)]
            prometheus_client_tracker: PrometheusClientTracker::start(gauge_labels),
        }
    }

    #[allow(unused_variables)]
    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        #[cfg(metrics)]
        self.metrics_tracker
            .finish(counter_labels, histogram_labels);
        #[cfg(opentelemetry)]
        self.opentelemetry_tracker
            .finish(counter_labels, histogram_labels);
        #[cfg(prometheus)]
        self.prometheus_tracker
            .finish(counter_labels, histogram_labels);
        #[cfg(prometheus_client)]
        self.prometheus_client_tracker
            .finish(counter_labels, histogram_labels);
    }

    #[cfg(debug_assertions)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]) {
        #[cfg(metrics)]
        MetricsTracker::intitialize_metrics(function_descriptions);
        #[cfg(opentelemetry)]
        OpenTelemetryTracker::intitialize_metrics(function_descriptions);
        #[cfg(prometheus)]
        PrometheusTracker::intitialize_metrics(function_descriptions);
        #[cfg(prometheus_client)]
        PrometheusClientTracker::intitialize_metrics(function_descriptions);
    }
}
