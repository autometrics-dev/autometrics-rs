use super::TrackMetrics;
#[cfg(debug_assertions)]
use crate::__private::FunctionDescription;
#[cfg(exemplars)]
use crate::exemplars::get_exemplar;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};
use crate::{constants::*, settings::get_settings};
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

static METRICS: Lazy<&Metrics> = Lazy::new(|| &get_settings().prometheus_client_metrics);

pub(crate) fn initialize_registry(mut registry: Registry) -> (Registry, Metrics) {
    let counter = Family::<CounterLabels, CounterType>::default();
    registry.register(
        // Remove the _total suffix from the counter name
        // because the library adds it automatically
        COUNTER_NAME_PROMETHEUS.replace("_total", ""),
        COUNTER_DESCRIPTION,
        counter.clone(),
    );

    let histogram = Family::<HistogramLabels, HistogramType>::new_with_constructor(|| {
        HistogramType::new(get_settings().histogram_buckets.iter().copied())
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
}

pub(crate) struct Metrics {
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

    #[cfg(debug_assertions)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]) {
        for function in function_descriptions {
            METRICS
                .counter
                .get_or_create(&CounterLabels::from(function))
                .inc_by(
                    0,
                    #[cfg(exemplars)]
                    None,
                );
        }
    }
}
