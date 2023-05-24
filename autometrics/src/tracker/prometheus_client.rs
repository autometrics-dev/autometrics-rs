use super::TrackMetrics;
#[cfg(feature = "exemplars-tracing")]
use crate::integrations::tracing::{get_exemplar, TraceLabel};
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};
use crate::{constants::*, HISTOGRAM_BUCKETS};
use once_cell::sync::Lazy;
#[cfg(feature = "exemplars-tracing")]
use prometheus_client::metrics::exemplar::{CounterWithExemplar, HistogramWithExemplars};
#[cfg(not(feature = "exemplars-tracing"))]
use prometheus_client::metrics::{counter::Counter, histogram::Histogram};
use prometheus_client::metrics::{family::Family, gauge::Gauge};
use prometheus_client::registry::Registry;
use std::time::Instant;

#[cfg(feature = "exemplars-tracing")]
type CounterType = CounterWithExemplar<TraceLabel>;
#[cfg(not(feature = "exemplars-tracing"))]
type CounterType = Counter;

#[cfg(feature = "exemplars-tracing")]
type HistogramType = HistogramWithExemplars<TraceLabel>;
#[cfg(not(feature = "exemplars-tracing"))]
type HistogramType = Histogram;

static REGISTRY_AND_METRICS: Lazy<(Registry, Metrics)> = Lazy::new(|| {
    let mut registry = <Registry>::default();

    let counter = Family::<CounterLabels, CounterType>::default();
    registry.register(
        COUNTER_NAME_PROMETHEUS,
        COUNTER_DESCRIPTION,
        counter.clone(),
    );

    let histogram = Family::<HistogramLabels, HistogramType>::new_with_constructor(|| {
        HistogramType::new(HISTOGRAM_BUCKETS.into_iter())
    });
    registry.register(
        HISTOGRAM_NAME_PROMETHEUS,
        HISTOGRAM_DESCRIPTION,
        histogram.clone(),
    );

    let gauge = Family::<GaugeLabels, Gauge>::default();
    registry.register(GAUGE_NAME_PROMETHEUS, GAUGE_DESCRIPTION, gauge.clone());

    let build_info = Family::<BuildInfoLabels, Gauge>::default();
    registry.register(BUILD_INFO_NAME, BUILD_INFO_DESCRIPTION, build_info.clone());

    (
        registry,
        Metrics {
            counter,
            histogram,
            gauge,
            build_info,
        },
    )
});
/// The [`Registry`] used to collect metrics when the `prometheus-client` feature is enabled
pub static REGISTRY: Lazy<&Registry> = Lazy::new(|| &REGISTRY_AND_METRICS.0);
static METRICS: Lazy<&Metrics> = Lazy::new(|| &REGISTRY_AND_METRICS.1);

struct Metrics {
    counter: Family<CounterLabels, CounterType>,
    histogram: Family<HistogramLabels, HistogramType>,
    gauge: Family<GaugeLabels, Gauge>,
    build_info: Family<BuildInfoLabels, Gauge>,
}

pub struct PrometheusClientTracker {
    gauge_labels: Option<GaugeLabels>,
    start_time: Instant,
}

impl TrackMetrics for PrometheusClientTracker {
    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        METRICS.build_info.get_or_create(&build_info_labels).set(1);
    }

    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        if let Some(gauge_labels) = gauge_labels {
            METRICS.gauge.get_or_create(&gauge_labels).inc();
        }
        Self {
            gauge_labels: gauge_labels.cloned(),
            start_time: Instant::now(),
        }
    }

    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        #[cfg(feature = "exemplars-tracing")]
        let exemplar = get_exemplar();

        let counter = METRICS.counter.get_or_create(&counter_labels);
        #[cfg(feature = "exemplars-tracing")]
        counter.inc_by(1, exemplar.clone());
        #[cfg(not(feature = "exemplars-tracing"))]
        counter.inc();

        let histogram = METRICS.histogram.get_or_create(&histogram_labels);
        let duration = self.start_time.elapsed().as_secs_f64();
        #[cfg(feature = "exemplars-tracing")]
        histogram.observe(duration, exemplar);
        #[cfg(not(feature = "exemplars-tracing"))]
        histogram.observe(duration);

        if let Some(gauge_labels) = self.gauge_labels {
            METRICS.gauge.get_or_create(&gauge_labels).dec();
        }
    }
}
