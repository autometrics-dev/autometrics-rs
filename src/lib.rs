mod result_labels;

pub use autometrics_macros::autometrics;

// Not public API.
#[doc(hidden)]
pub mod __private {
    use opentelemetry::metrics::{Histogram, UpDownCounter};
    use opentelemetry::{global, Key, KeyValue, Value};

    pub use crate::result_labels::*;
    pub use const_format::str_replace;
    pub use opentelemetry::Context;

    pub fn register_histogram(name: &'static str) -> Histogram<f64> {
        global::meter("")
            .f64_histogram(name)
            .with_description("Autometrics histogram for tracking function calls")
            .init()
    }

    pub fn create_concurrency_tracker(name: &'static str, labels: [KeyValue; 2]) -> GaugeGuard {
        let counter = global::meter("")
            .i64_up_down_counter(name)
            .with_description("Autometrics gauge for tracking concurrent function calls")
            .init();
        let context = Context::current();
        counter.add(&context, 1, &labels);

        GaugeGuard {
            counter,
            labels,
            context,
        }
    }

    /// Decrease the gauge by 1 when the guard is dropped
    pub struct GaugeGuard {
        counter: UpDownCounter<i64>,
        labels: [KeyValue; 2],
        context: Context,
    }

    impl Drop for GaugeGuard {
        fn drop(&mut self) {
            self.counter.add(&self.context, -1, &self.labels);
        }
    }

    pub fn create_labels(function_name: &'static str, module: &'static str) -> [KeyValue; 2] {
        [
            KeyValue {
                key: Key::from_static_str("function"),
                value: Value::String(function_name.into()),
            },
            KeyValue {
                key: Key::from_static_str("module"),
                value: Value::String(module.into()),
            },
        ]
    }

    pub fn create_labels_with_result(
        function_name: &'static str,
        module: &'static str,
        result: &'static str,
    ) -> [KeyValue; 3] {
        [
            KeyValue {
                key: Key::from_static_str("function"),
                value: Value::String(function_name.into()),
            },
            KeyValue {
                key: Key::from_static_str("module"),
                value: Value::String(module.into()),
            },
            KeyValue {
                key: Key::from_static_str("result"),
                value: Value::String(result.into()),
            },
        ]
    }
}
