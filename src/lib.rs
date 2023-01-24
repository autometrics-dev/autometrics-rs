mod labels;
mod prometheus;

#[cfg(feature = "prometheus-exporter")]
pub use self::prometheus::*;
pub use autometrics_macros::autometrics;

// Not public API.
#[doc(hidden)]
pub mod __private {
    use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
    use opentelemetry::{global, KeyValue};
    use std::{cell::RefCell, thread_local};
    use tokio::task::LocalKey;

    pub use crate::labels::*;
    pub use const_format::str_replace;
    pub use opentelemetry::Context;

    /// Task-local value used for tracking which function called the current function
    pub static CALLER: LocalKey<&'static str> = {
        // This does the same thing as the tokio::thread_local macro with the exception that
        // it initializes the value with the empty string.
        // The tokio macro does not allow you to get the value before setting it.
        // However, in our case, we want it to simply return the empty string rather than panicking.
        thread_local! {
            static CALLER_KEY: RefCell<Option<&'static str>> = const { RefCell::new(Some("")) };
        }

        LocalKey { inner: CALLER_KEY }
    };

    pub fn register_counter(name: &'static str) -> Counter<f64> {
        global::meter("")
            .f64_counter(name)
            .with_description("Autometrics counter for tracking function calls")
            .init()
    }

    pub fn register_histogram(name: &'static str) -> Histogram<f64> {
        global::meter("")
            .f64_histogram(name)
            .with_description("Autometrics histogram for tracking function latency")
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
