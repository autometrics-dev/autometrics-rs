#[cfg(debug_assertions)]
use crate::__private::FunctionDescription;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels, ResultLabel};
use crate::{constants::*, settings::get_settings, tracker::TrackMetrics};
use once_cell::sync::Lazy;
use prometheus::core::{AtomicI64, GenericGauge};
use prometheus::{
    histogram_opts, register_histogram_vec_with_registry, register_int_counter_vec_with_registry,
    register_int_gauge_vec_with_registry, HistogramVec, IntCounterVec, IntGaugeVec,
};
use std::{sync::Once, time::Instant};

static SET_BUILD_INFO: Once = Once::new();

static COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec_with_registry!(
        COUNTER_NAME_PROMETHEUS,
        COUNTER_DESCRIPTION,
        &[
            FUNCTION_KEY,
            MODULE_KEY,
            SERVICE_NAME_KEY_PROMETHEUS,
            CALLER_FUNCTION_PROMETHEUS,
            CALLER_MODULE_PROMETHEUS,
            RESULT_KEY,
            OK_KEY,
            ERROR_KEY,
            OBJECTIVE_NAME_PROMETHEUS,
            OBJECTIVE_PERCENTILE_PROMETHEUS,
        ],
        get_settings().prometheus_registry.clone()
    )
    .expect("Failed to register function_calls_count_total counter")
});
static HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = histogram_opts!(
        HISTOGRAM_NAME_PROMETHEUS,
        HISTOGRAM_DESCRIPTION,
        // The Prometheus crate uses different histogram buckets by default
        // (and these are configured when creating a histogram rather than
        // when configuring the registry or exporter, like in the other crates)
        // so we need to pass these in here
        get_settings().histogram_buckets.clone()
    );
    register_histogram_vec_with_registry!(
        opts,
        &[
            FUNCTION_KEY,
            MODULE_KEY,
            SERVICE_NAME_KEY_PROMETHEUS,
            OBJECTIVE_NAME_PROMETHEUS,
            OBJECTIVE_PERCENTILE_PROMETHEUS,
            OBJECTIVE_LATENCY_THRESHOLD_PROMETHEUS
        ],
        get_settings().prometheus_registry.clone()
    )
    .expect("Failed to register function_calls_duration histogram")
});
static GAUGE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec_with_registry!(
        GAUGE_NAME_PROMETHEUS,
        GAUGE_DESCRIPTION,
        &[FUNCTION_KEY, MODULE_KEY, SERVICE_NAME_KEY_PROMETHEUS],
        get_settings().prometheus_registry.clone()
    )
    .expect("Failed to register function_calls_concurrent gauge")
});
static BUILD_INFO: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec_with_registry!(
        BUILD_INFO_NAME,
        BUILD_INFO_DESCRIPTION,
        &[
            COMMIT_KEY,
            VERSION_KEY,
            BRANCH_KEY,
            SERVICE_NAME_KEY_PROMETHEUS,
            REPO_URL_KEY_PROMETHEUS,
            REPO_PROVIDER_KEY_PROMETHEUS,
            AUTOMETRICS_VERSION_KEY_PROMETHEUS,
        ],
        get_settings().prometheus_registry.clone()
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
            let gauge = GAUGE.with_label_values(&[
                gauge_labels.function,
                gauge_labels.module,
                gauge_labels.service_name,
            ]);
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

        let counter_labels = counter_labels_to_prometheus_vec(counter_labels);
        COUNTER.with_label_values(&counter_labels).inc();

        HISTOGRAM
            .with_label_values(&[
                histogram_labels.function,
                histogram_labels.module,
                histogram_labels.service_name,
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
                    build_info_labels.service_name,
                    build_info_labels.repo_url,
                    build_info_labels.repo_provider,
                    AUTOMETRICS_SPEC_TARGET
                ])
                .set(1);
        });
    }

    #[cfg(debug_assertions)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]) {
        for function in function_descriptions {
            let labels = counter_labels_to_prometheus_vec(&CounterLabels::from(function));
            COUNTER.with_label_values(&labels).inc_by(0);
        }
    }
}

/// Put the label values in the same order as the keys in the counter definition
fn counter_labels_to_prometheus_vec(counter_labels: &CounterLabels) -> [&'static str; 10] {
    [
        counter_labels.function,
        counter_labels.module,
        counter_labels.service_name,
        counter_labels.caller_function,
        counter_labels.caller_module,
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
    ]
}
