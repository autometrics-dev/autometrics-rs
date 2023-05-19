use super::TrackMetrics;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};
use crate::{constants::*, HISTOGRAM_BUCKETS};
use once_cell::sync::Lazy;
use prometheus_client::metrics::{
    counter::Counter, family::Family, gauge::Gauge, histogram::Histogram,
};
use prometheus_client::registry::Registry;
use std::time::Instant;

static REGISTRY_AND_METRICS: Lazy<(Registry, Metrics)> = Lazy::new(|| {
    let mut registry = <Registry>::default();

    let counter = Family::<CounterLabels, Counter>::default();
    registry.register(
        COUNTER_NAME_PROMETHEUS,
        COUNTER_DESCRIPTION,
        counter.clone(),
    );

    let histogram = Family::<HistogramLabels, Histogram>::new_with_constructor(|| {
        Histogram::new(HISTOGRAM_BUCKETS.into_iter())
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

struct Metrics {
    counter: Family<CounterLabels, Counter>,
    histogram: Family<HistogramLabels, Histogram>,
    gauge: Family<GaugeLabels, Gauge>,
    build_info: Family<BuildInfoLabels, Gauge>,
}

pub struct PrometheusClientTracker {
    gauge_labels: Option<GaugeLabels>,
    start_time: Instant,
}

impl TrackMetrics for PrometheusClientTracker {
    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        REGISTRY_AND_METRICS
            .1
            .build_info
            .get_or_create(&build_info_labels)
            .set(1);
    }

    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        if let Some(gauge_labels) = gauge_labels {
            REGISTRY_AND_METRICS
                .1
                .gauge
                .get_or_create(&gauge_labels)
                .inc();
        }
        Self {
            gauge_labels: gauge_labels.cloned(),
            start_time: Instant::now(),
        }
    }

    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        REGISTRY_AND_METRICS
            .1
            .counter
            .get_or_create(&counter_labels)
            .inc();
        REGISTRY_AND_METRICS
            .1
            .histogram
            .get_or_create(&histogram_labels)
            .observe(self.start_time.elapsed().as_secs_f64());
        if let Some(gauge_labels) = self.gauge_labels {
            REGISTRY_AND_METRICS
                .1
                .gauge
                .get_or_create(&gauge_labels)
                .dec();
        }
    }
}
