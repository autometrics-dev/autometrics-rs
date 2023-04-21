use super::TrackMetrics;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};
use crate::{constants::*, HISTOGRAM_BUCKETS};
use once_cell::sync::Lazy;
use prometheus_client::metrics::{
    counter::Counter, family::Family, gauge::Gauge, histogram::Histogram,
};
use prometheus_client::registry::Registry;

pub static PROMETHEUS_CLIENT_REGISTRY: Lazy<Registry> = Lazy::new(|| <Registry>::default());

static METRICS: Lazy<Metrics> = Lazy::new(|| {
    let counter = Family::<CounterLabels, Counter>::default();
    PROMETHEUS_CLIENT_REGISTRY.register(COUNTER_NAME, COUNTER_DESCRIPTION, counter.clone());

    let histogram = Family::<HistogramLabels, Histogram>::new_with_constructor(|| {
        Histogram::new(HISTOGRAM_BUCKETS.into_iter())
    });
    PROMETHEUS_CLIENT_REGISTRY.register(HISTOGRAM_NAME, HISTOGRAM_DESCRIPTION, histogram.clone());

    let gauge = Family::<GaugeLabels, Gauge>::default();
    PROMETHEUS_CLIENT_REGISTRY.register(GAUGE_NAME, GAUGE_DESCRIPTION, gauge.clone());

    let build_info = Family::<BuildInfoLabels, Gauge>::default();
    PROMETHEUS_CLIENT_REGISTRY.register(
        BUILD_INFO_NAME,
        BUILD_INFO_DESCRIPTION,
        build_info.clone(),
    );

    Metrics {
        counter,
        histogram,
        gauge,
        build_info,
    }
});

struct Metrics {
    counter: Family<CounterLabels, Counter>,
    histogram: Family<HistogramLabels, Histogram>,
    gauge: Family<GaugeLabels, Gauge>,
    build_info: Family<BuildInfoLabels, Gauge>,
}

struct PrometheusClientTracker {}

impl TrackMetrics for PrometheusClientTracker {
    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        todo!()
    }

    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        todo!()
    }

    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        todo!()
    }
}
