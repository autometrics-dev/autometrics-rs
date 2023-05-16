use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels, ResultLabel};
use crate::{constants::*, tracker::TrackMetrics, HISTOGRAM_BUCKETS};
use const_format::{formatcp, str_replace};
use once_cell::sync::Lazy;
use prometheus::core::{AtomicI64, GenericGauge};
use prometheus::{
    histogram_opts, register_histogram_vec, register_int_counter_vec, register_int_gauge_vec,
    HistogramVec, IntCounterVec, IntGaugeVec,
};
use std::{sync::Once, time::Instant};

static SET_BUILD_INFO: Once = Once::new();

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
            OBJECTIVE_NAME_PROMETHEUS,
            OBJECTIVE_PERCENTILE_PROMETHEUS,
        ]
    )
    .expect(formatcp!(
        "Failed to register {COUNTER_NAME_PROMETHEUS} counter"
    ))
});
static HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = histogram_opts!(
        HISTOGRAM_NAME_PROMETHEUS,
        HISTOGRAM_DESCRIPTION,
        // The Prometheus crate uses different histogram buckets by default
        // (and these are configured when creating a histogram rather than
        // when configuring the registry or exporter, like in the other crates)
        // so we need to pass these in here
        HISTOGRAM_BUCKETS.to_vec()
    );
    register_histogram_vec!(
        opts,
        &[
            FUNCTION_KEY,
            MODULE_KEY,
            OBJECTIVE_NAME_PROMETHEUS,
            OBJECTIVE_PERCENTILE_PROMETHEUS,
            OBJECTIVE_LATENCY_THRESHOLD_PROMETHEUS
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
static BUILD_INFO: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        BUILD_INFO_NAME,
        BUILD_INFO_DESCRIPTION,
        &[COMMIT_KEY, VERSION_KEY, BRANCH_KEY]
    )
    .expect("Failed to register build_info counter")
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
                    match counter_labels.result {
                        Some(ResultLabel::Ok) => OK_KEY,
                        Some(ResultLabel::Error) => ERROR_KEY,
                        None => "",
                    },
                    counter_labels.ok.unwrap_or_default(),
                    counter_labels.error.unwrap_or_default(),
                    counter_labels.objective_name.unwrap_or_default(),
                    counter_labels
                        .objective_percentile
                        .as_ref()
                        .map(|p| p.as_str())
                        .unwrap_or_default(),
                ],
            )
            .inc();

        HISTOGRAM
            .with_label_values(&[
                histogram_labels.function,
                histogram_labels.module,
                histogram_labels.objective_name.unwrap_or_default(),
                histogram_labels
                    .objective_percentile
                    .as_ref()
                    .map(|p| p.as_str())
                    .unwrap_or_default(),
                histogram_labels
                    .objective_latency_threshold
                    .as_ref()
                    .map(|p| p.as_str())
                    .unwrap_or_default(),
            ])
            .observe(duration);

        if let Some(gauge) = self.gauge {
            gauge.dec();
        }
    }

    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        SET_BUILD_INFO.call_once(|| {
            BUILD_INFO
                .with_label_values(&[
                    build_info_labels.commit,
                    build_info_labels.version,
                    build_info_labels.branch,
                ])
                .set(1);
        });
    }
}
