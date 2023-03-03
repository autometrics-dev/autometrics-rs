use crate::labels::{CounterLabels, GaugeLabels, HistogramLabels};
use crate::{constants::*, tracker::TrackMetrics};
use const_format::{formatcp, str_replace};
use once_cell::sync::Lazy;
use prometheus::{
    core::{AtomicI64, GenericGauge},
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, HistogramVec,
    IntCounterVec, IntGaugeVec,
};
use std::time::Instant;

const COUNTER_NAME_PROMETHEUS: &str = str_replace!(COUNTER_NAME, ".", "_");
const HISTOGRAM_NAME_PROMETHEUS: &str = str_replace!(HISTOGRAM_NAME, ".", "_");
const GAUGE_NAME_PROMETHEUS: &str = str_replace!(GAUGE_NAME, ".", "_");

static COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        COUNTER_NAME_PROMETHEUS,
        COUNTER_DESCRIPTION,
        &[
            FUNCTION_KEY,
            MODULE_KEY,
            CALLER_KEY,
            RESULT_KEY,
            OK_KEY,
            ERROR_KEY,
            SLO_NAME,
            OBJECTIVE,
        ]
    )
    .expect(formatcp!(
        "Failed to register {COUNTER_NAME_PROMETHEUS} counter"
    ))
});
static HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        HISTOGRAM_NAME_PROMETHEUS,
        HISTOGRAM_DESCRIPTION,
        &[
            FUNCTION_KEY,
            MODULE_KEY,
            SLO_NAME,
            OBJECTIVE,
            TARGET_LATENCY
        ]
    )
    .expect("Failed to register function_calls_duration histogram")
});
static GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        GAUGE_NAME_PROMETHEUS,
        GAUGE_DESCRIPTION,
        &[FUNCTION_KEY, MODULE_KEY]
    )
    .expect("Failed to register function_calls_concurrent gauge")
});

pub struct PrometheusTracker {
    start: Instant,
    gauge: Option<GenericGauge<AtomicI64>>,
}

impl TrackMetrics for PrometheusTracker {
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        let gauge = if let Some(gauge_labels) = gauge_labels {
            let gauge = GAUGE.with_label_values(&[gauge_labels.function, gauge_labels.module]);
            gauge.inc();
            Some(gauge)
        } else {
            None
        };

        Self {
            start: Instant::now(),
            gauge,
        }
    }

    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        let duration = self.start.elapsed().as_secs_f64();

        COUNTER
            .with_label_values(
                // Put the label values in the same order as the keys in the counter definition
                &[
                    counter_labels.function,
                    counter_labels.module,
                    counter_labels.caller,
                    counter_labels.result.unwrap_or_default().0,
                    if let Some((OK_KEY, Some(return_value_type))) = counter_labels.result {
                        return_value_type
                    } else {
                        ""
                    },
                    if let Some((ERROR_KEY, Some(return_value_type))) = counter_labels.result {
                        return_value_type
                    } else {
                        ""
                    },
                    counter_labels.objective.unwrap_or_default().0,
                    counter_labels.objective.unwrap_or_default().1,
                ],
            )
            .inc();

        HISTOGRAM
            .with_label_values(&[
                histogram_labels.function,
                histogram_labels.module,
                histogram_labels.objective.unwrap_or_default().0,
                histogram_labels.objective.unwrap_or_default().1,
                histogram_labels.objective.unwrap_or_default().2,
            ])
            .observe(duration);

        if let Some(gauge) = self.gauge {
            gauge.dec();
        }
    }
}
