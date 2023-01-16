mod labels;

pub use autometrics_macros::autometrics;

// Not public API.
#[doc(hidden)]
pub mod __private {
    use opentelemetry::metrics::{Histogram, UpDownCounter};
    use opentelemetry::{global, KeyValue};

    pub use crate::labels::*;
    pub use const_format::str_replace;
    pub use opentelemetry::Context;

    pub fn register_histogram(name: &'static str) -> Histogram<f64> {
        global::meter("")
            .f64_histogram(name)
            .with_description("Autometrics histogram for tracking function calls")
            .init()
    }

    /// Increment the gauge by 1 and then decrement it again when the returned value is dropped
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
}
