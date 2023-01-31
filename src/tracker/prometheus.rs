use crate::{constants::*, labels::Label, tracker::TrackMetrics};
use const_format::{formatcp, str_replace};
use once_cell::sync::Lazy;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, HistogramVec,
    IntCounterVec, IntGaugeVec,
};
use std::{collections::HashMap, time::Instant};

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
        &[FUNCTION_KEY, MODULE_KEY]
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
    module: &'static str,
    function: &'static str,
    start: Instant,
    track_concurrency: bool,
}

impl TrackMetrics for PrometheusTracker {
    fn function(&self) -> &'static str {
        self.function
    }

    fn module(&self) -> &'static str {
        self.module
    }

    fn start(function: &'static str, module: &'static str, track_concurrency: bool) -> Self {
        if track_concurrency {
            GAUGE.with_label_values(&[function, module]).inc();
        }

        Self {
            function,
            module,
            start: Instant::now(),
            track_concurrency,
        }
    }

    fn finish(self, counter_labels: &[Label]) {
        let duration = self.start.elapsed().as_secs_f64();

        let labels: HashMap<&str, &str> = counter_labels.iter().map(|(k, v)| (*k, *v)).collect();
        COUNTER
            .with_label_values(
                // Put the label values in the same order as the keys in the counter definition
                &[
                    labels.get(FUNCTION_KEY).unwrap_or(&""),
                    labels.get(MODULE_KEY).unwrap_or(&""),
                    labels.get(CALLER_KEY).unwrap_or(&""),
                    labels.get(RESULT_KEY).unwrap_or(&""),
                    labels.get(OK_KEY).unwrap_or(&""),
                    labels.get(ERROR_KEY).unwrap_or(&""),
                ],
            )
            .inc();

        HISTOGRAM
            .with_label_values(&[self.function, self.module])
            .observe(duration);

        if self.track_concurrency {
            GAUGE.with_label_values(&[self.function, self.module]).dec();
        }
    }
}
