use crate::labels::create_labels;
use opentelemetry::{global, metrics::UpDownCounter, Context, KeyValue};
use std::time::Instant;

/// Tracks the number of function calls, concurrent calls, and latency
pub struct AutometricsTracker {
    pub module: &'static str,
    pub function: &'static str,
    concurrency_tracker: UpDownCounter<i64>,
    function_and_module_labels: [KeyValue; 2],
    start: Instant,
    context: Context,
}

impl AutometricsTracker {
    pub fn start(function: &'static str, module: &'static str, gauge_name: &'static str) -> Self {
        let function_and_module_labels = create_labels(function, module);

        // Increase the number of concurrent requests
        let concurrency_tracker = global::meter("")
            .i64_up_down_counter(gauge_name)
            .with_description("Autometrics gauge for tracking concurrent function calls")
            .init();
        let context = Context::current();
        concurrency_tracker.add(&context, 1, &function_and_module_labels);

        Self {
            function,
            module,
            function_and_module_labels,
            concurrency_tracker,
            start: Instant::now(),
            context,
        }
    }

    pub fn finish(
        self,
        histogram_name: &'static str,
        counter_name: &'static str,
        counter_labels: &[KeyValue],
    ) {
        let duration = self.start.elapsed().as_secs_f64();

        // Track the function calls
        let counter = global::meter("")
            .f64_counter(counter_name)
            .with_description("Autometrics counter for tracking function calls")
            .init();
        counter.add(&self.context, 1.0, &counter_labels);

        // Track the latency
        let histogram = global::meter("")
            .f64_histogram(histogram_name)
            .with_description("Autometrics histogram for tracking function latency")
            .init();
        histogram.record(&self.context, duration, &self.function_and_module_labels);

        // Decrease the number of concurrent requests
        self.concurrency_tracker
            .add(&self.context, -1, &self.function_and_module_labels);
    }
}
