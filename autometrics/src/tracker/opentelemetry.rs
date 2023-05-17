use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels, Label};
use crate::{constants::*, tracker::TrackMetrics};
use once_cell::sync::Lazy;
use opentelemetry_api::metrics::{Counter, Histogram, UpDownCounter};
use opentelemetry_api::{global, Context, KeyValue};
use std::{sync::Once, time::Instant};

static SET_BUILD_INFO: Once = Once::new();
static CONCURRENCY_TRACKER: Lazy<UpDownCounter<i64>> = Lazy::new(|| {
    global::meter("autometrics")
        .i64_up_down_counter(GAUGE_NAME)
        .with_description(GAUGE_DESCRIPTION)
        .init()
});
static COUNTER: Lazy<Counter<f64>> = Lazy::new(|| {
    global::meter("autometrics")
        .f64_counter(COUNTER_NAME)
        .with_description(COUNTER_DESCRIPTION)
        .init()
});
static HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    global::meter("autometrics")
        .f64_histogram(HISTOGRAM_NAME)
        .with_description(HISTOGRAM_DESCRIPTION)
        .init()
});

/// Tracks the number of function calls, concurrent calls, and latency
pub struct OpenTelemetryTracker {
    gauge_labels: Option<Vec<KeyValue>>,
    start: Instant,
    context: Context,
}

impl TrackMetrics for OpenTelemetryTracker {
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        let context = Context::current();

        let gauge_labels = if let Some(gauge_labels) = gauge_labels {
            let gauge_labels = to_key_values(gauge_labels.to_array());
            // Increase the number of concurrent requests
            CONCURRENCY_TRACKER.add(&context, 1, &gauge_labels);
            Some(gauge_labels)
        } else {
            None
        };

        Self {
            gauge_labels,
            start: Instant::now(),
            context,
        }
    }

    fn finish<'a>(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        let duration = self.start.elapsed().as_secs_f64();

        // Track the function calls
        let counter_labels = to_key_values(counter_labels.to_vec());
        COUNTER.add(&self.context, 1.0, &counter_labels);

        // Track the latency
        let histogram_labels = to_key_values(histogram_labels.to_vec());
        HISTOGRAM.record(&self.context, duration, &histogram_labels);

        // Decrease the number of concurrent requests
        if let Some(gauge_labels) = self.gauge_labels {
            CONCURRENCY_TRACKER.add(&self.context, -1, &gauge_labels);
        }
    }

    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        SET_BUILD_INFO.call_once(|| {
            let build_info_labels = to_key_values(build_info_labels.to_vec());
            let build_info = global::meter("autometrics")
                .f64_up_down_counter(BUILD_INFO_NAME)
                .with_description(BUILD_INFO_DESCRIPTION)
                .init();
            build_info.add(&Context::current(), 1.0, &build_info_labels);
        });
    }
}

fn to_key_values(labels: impl IntoIterator<Item = Label>) -> Vec<KeyValue> {
    labels
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect()
}
