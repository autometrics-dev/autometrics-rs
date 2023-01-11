mod result_labels;
#[cfg(test)]
mod tests;

pub use autometrics_macros::autometrics;

// Not public API.
#[doc(hidden)]
pub mod __private {
    use opentelemetry::{global, metrics::Histogram, Key, KeyValue, Value};

    pub use crate::result_labels::*;
    pub use const_format::str_replace;
    pub use opentelemetry::Context;

    pub fn histogram(name: &'static str) -> Histogram<f64> {
        global::meter("")
            .f64_histogram(name)
            .with_description("Autometrics histogram for tracking function calls")
            .init()
    }

    pub fn labels(function_name: &'static str, module: &'static str) -> [KeyValue; 2] {
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

    pub fn labels_with_result(
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
