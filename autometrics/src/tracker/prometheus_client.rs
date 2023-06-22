use super::TrackMetrics;
#[cfg(exemplars)]
use crate::exemplars::get_exemplar;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};
use crate::{constants::*, HISTOGRAM_BUCKETS};
use once_cell::sync::Lazy;
use prometheus_client::metrics::{family::Family, gauge::Gauge};
use prometheus_client::registry::{Registry, Unit};
use std::time::Instant;

#[cfg(exemplars)]
type CounterType =
    prometheus_client::metrics::exemplar::CounterWithExemplar<Vec<(&'static str, String)>>;
#[cfg(not(exemplars))]
type CounterType = prometheus_client::metrics::counter::Counter;

#[cfg(exemplars)]
type HistogramType =
    prometheus_client::metrics::exemplar::HistogramWithExemplars<Vec<(&'static str, String)>>;
#[cfg(not(exemplars))]
type HistogramType = prometheus_client::metrics::histogram::Histogram;

static REGISTRY_AND_METRICS: Lazy<(Registry, Metrics)> = Lazy::new(|| {
    let mut registry = <Registry>::default();

    let counter = Family::<CounterLabels, CounterType>::default();
    registry.register(
        // Remove the _total suffix from the counter name
        // because the library adds it automatically
        COUNTER_NAME_PROMETHEUS.replace("_total", ""),
        COUNTER_DESCRIPTION,
        counter.clone(),
    );

    let histogram = Family::<HistogramLabels, HistogramType>::new_with_constructor(|| {
        HistogramType::new(HISTOGRAM_BUCKETS.into_iter())
    });
    registry.register_with_unit(
        // This also adds the _seconds suffix to the histogram name automatically
        HISTOGRAM_NAME_PROMETHEUS.replace("_seconds", ""),
        HISTOGRAM_DESCRIPTION,
        Unit::Seconds,
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
        METRICS.build_info.get_or_create(build_info_labels).set(1);
    }

    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        if let Some(gauge_labels) = gauge_labels {
            METRICS.gauge.get_or_create(gauge_labels).inc();
        }
        Self {
            gauge_labels: gauge_labels.cloned(),
            start_time: Instant::now(),
        }
    }

    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        #[cfg(exemplars)]
        let exemplar = get_exemplar().map(|exemplar| exemplar.into_iter().collect::<Vec<_>>());

        METRICS.counter.get_or_create(counter_labels).inc_by(
            1,
            #[cfg(exemplars)]
            exemplar.clone(),
        );

        METRICS.histogram.get_or_create(histogram_labels).observe(
            self.start_time.elapsed().as_secs_f64(),
            #[cfg(exemplars)]
            exemplar,
        );

        if let Some(gauge_labels) = &self.gauge_labels {
            METRICS.gauge.get_or_create(gauge_labels).dec();
        }
    }
}
