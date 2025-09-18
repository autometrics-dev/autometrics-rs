#[cfg(debug_assertions)]
use crate::__private::FunctionDescription;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels, Label};
use crate::{constants::*, tracker::TrackMetrics};
use once_cell::sync::Lazy;
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
use opentelemetry::{global, KeyValue};
use std::{sync::Once, time::Instant};

static SET_BUILD_INFO: Once = Once::new();
const METER_NAME: &str = "autometrics";
static COUNTER: Lazy<Counter<u64>> = Lazy::new(|| {
    global::meter(METER_NAME)
        .u64_counter(COUNTER_NAME)
        .with_description(COUNTER_DESCRIPTION)
        .build()
});
static HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    // Note that the unit needs to be written as "s" rather than "seconds"
    // or it will not be included in the metric name
    // https://github.com/open-telemetry/opentelemetry-rust/issues/1173
    global::meter(METER_NAME)
        .f64_histogram(HISTOGRAM_NAME)
        .with_unit("s")
        .with_description(HISTOGRAM_DESCRIPTION)
        .build()
});
static GAUGE: Lazy<UpDownCounter<i64>> = Lazy::new(|| {
    global::meter(METER_NAME)
        .i64_up_down_counter(GAUGE_NAME)
        .with_description(GAUGE_DESCRIPTION)
        .build()
});

/// Tracks the number of function calls, concurrent calls, and latency
pub struct OpenTelemetryTracker {
    gauge_labels: Option<Vec<KeyValue>>,
    start: Instant,
}

impl TrackMetrics for OpenTelemetryTracker {
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        let gauge_labels = if let Some(gauge_labels) = gauge_labels {
            let gauge_labels = to_key_values(gauge_labels.to_array());
            // Increase the number of concurrent requests
            GAUGE.add(1, &gauge_labels);
            Some(gauge_labels)
        } else {
            None
        };

        Self {
            gauge_labels,
            start: Instant::now(),
        }
    }

    fn finish<'a>(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        let duration = self.start.elapsed().as_secs_f64();

        // Track the function calls
        let counter_labels = to_key_values(counter_labels.to_vec());
        COUNTER.add(1, &counter_labels);

        // Track the latency
        let histogram_labels = to_key_values(histogram_labels.to_vec());
        HISTOGRAM.record(duration, &histogram_labels);

        // Decrease the number of concurrent requests
        if let Some(gauge_labels) = self.gauge_labels {
            GAUGE.add(-1, &gauge_labels);
        }
    }

    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        SET_BUILD_INFO.call_once(|| {
            let build_info_labels = to_key_values(build_info_labels.to_vec());
            let build_info = global::meter(METER_NAME)
                .f64_up_down_counter(BUILD_INFO_NAME)
                .with_description(BUILD_INFO_DESCRIPTION)
                .build();
            build_info.add(1.0, &build_info_labels);
        });
    }

    #[cfg(debug_assertions)]
    fn intitialize_metrics(function_descriptions: &[FunctionDescription]) {
        for function in function_descriptions {
            let labels = &to_key_values(CounterLabels::from(function).to_vec());
            COUNTER.add(0, labels);
        }
    }
}

fn to_key_values(labels: impl IntoIterator<Item = Label>) -> Vec<KeyValue> {
    labels
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect()
}
