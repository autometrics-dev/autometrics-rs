use crate::labels::{CounterLabels, GaugeLabels, HistogramLabels, Label};
use crate::{constants::*, tracker::TrackMetrics};
use opentelemetry_api::{global, metrics::UpDownCounter, Context, KeyValue};
use std::time::Instant;

/// Tracks the number of function calls, concurrent calls, and latency
pub struct OpenTelemetryTracker {
    concurrency_tracker: Option<(UpDownCounter<i64>, Vec<KeyValue>)>,
    start: Instant,
    context: Context,
}

impl TrackMetrics for OpenTelemetryTracker {
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        let context = Context::current();

        let concurrency_tracker = if let Some(gauge_labels) = gauge_labels {
            let gauge_labels = to_key_values(gauge_labels.to_array());
            // Increase the number of concurrent requests
            let concurrency_tracker = global::meter("")
                .i64_up_down_counter(GAUGE_NAME)
                .with_description(GAUGE_DESCRIPTION)
                .init();
            concurrency_tracker.add(&context, 1, &gauge_labels);
            Some((concurrency_tracker, gauge_labels))
        } else {
            None
        };

        Self {
            concurrency_tracker,
            start: Instant::now(),
            context,
        }
    }

    fn finish<'a>(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        let duration = self.start.elapsed().as_secs_f64();

        // Track the function calls
        let counter = global::meter("")
            .f64_counter(COUNTER_NAME)
            .with_description(COUNTER_DESCRIPTION)
            .init();
        let counter_labels = to_key_values(counter_labels.to_vec());
        counter.add(&self.context, 1.0, &counter_labels);

        // Track the latency
        let histogram = global::meter("")
            .f64_histogram(HISTOGRAM_NAME)
            .with_description(HISTOGRAM_DESCRIPTION)
            .init();
        let histogram_labels = to_key_values(histogram_labels.to_array());
        histogram.record(&self.context, duration, &histogram_labels);

        // Decrease the number of concurrent requests
        if let Some((concurrency_tracker, gauge_labels)) = self.concurrency_tracker {
            concurrency_tracker.add(&self.context, -1, &gauge_labels);
        }
    }
}

fn to_key_values(labels: impl IntoIterator<Item = Label>) -> Vec<KeyValue> {
    labels
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect()
}
