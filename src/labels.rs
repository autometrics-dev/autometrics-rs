use opentelemetry::{Key, KeyValue, Value};
use std::ops::Deref;

const FUNCTION_KEY: Key = Key::from_static_str("function");
const MODULE_KEY: Key = Key::from_static_str("module");
const RESULT_KEY: Key = Key::from_static_str("result");

pub fn create_labels(function_name: &'static str, module: &'static str) -> [KeyValue; 2] {
    [
        KeyValue {
            key: FUNCTION_KEY,
            value: Value::String(function_name.into()),
        },
        KeyValue {
            key: MODULE_KEY,
            value: Value::String(module.into()),
        },
    ]
}

// The following is a convoluted way to figure out if the return type resolves to a Result
// or not. We cannot simply parse the code using syn to figure out if it's a Result
// because syn doesn't do type resolution and thus would count any renamed version
// of Result as a different type. Instead, we define two traits with intentionally
// conflicting method names and use a trick based on the order in which Rust resolves
// method names to return a different value based on whether the return value is
// a Result or anything else.
// This approach is based on dtolnay's answer to this question:
// https://users.rust-lang.org/t/how-to-check-types-within-macro/33803/5
// and this answer explains why it works:
// https://users.rust-lang.org/t/how-to-check-types-within-macro/33803/8

pub trait GetLabelsFromResult {
    fn __autometrics_get_labels(&self, function: &'static str, module: &'static str) -> LabelArray;
}

impl<T, E> GetLabelsFromResult for Result<T, E> {
    fn __autometrics_get_labels(&self, function: &'static str, module: &'static str) -> LabelArray {
        let (result, value_as_static_str) = match self {
            Ok(ok) => ("ok", ok.__autometrics_static_str()),
            Err(err) => ("error", err.__autometrics_static_str()),
        };

        let function_label = KeyValue {
            key: FUNCTION_KEY,
            value: Value::String(function.into()),
        };
        let module_label = KeyValue {
            key: MODULE_KEY,
            value: Value::String(module.into()),
        };
        let result_label = KeyValue {
            key: RESULT_KEY,
            value: Value::String(result.into()),
        };

        // Add another label for the return value if the type implements Into<&'static str>.
        // This is most likely useful for enums representing error (or potentially success) types.
        if let Some(value) = value_as_static_str {
            let value_label = KeyValue {
                key: Key::from_static_str(result),
                value: Value::String(snake_case(value).into()),
            };
            LabelArray::Four([function_label, module_label, result_label, value_label])
        } else {
            LabelArray::Three([function_label, module_label, result_label])
        }
    }
}

pub enum LabelArray {
    Three([KeyValue; 3]),
    Four([KeyValue; 4]),
}

impl Deref for LabelArray {
    type Target = [KeyValue];

    fn deref(&self) -> &Self::Target {
        match self {
            LabelArray::Three(l) => l,
            LabelArray::Four(l) => l,
        }
    }
}

pub trait GetLabels {
    fn __autometrics_get_labels(
        &self,
        function: &'static str,
        module: &'static str,
    ) -> [KeyValue; 2] {
        create_labels(function, module)
    }
}

impl<T> GetLabels for &T {}

// Implement for primitives
impl GetLabels for i8 {}
impl GetLabels for i16 {}
impl GetLabels for i32 {}
impl GetLabels for i64 {}
impl GetLabels for i128 {}
impl GetLabels for isize {}
impl GetLabels for u8 {}
impl GetLabels for u16 {}
impl GetLabels for u32 {}
impl GetLabels for u64 {}
impl GetLabels for u128 {}
impl GetLabels for usize {}
impl GetLabels for f32 {}
impl GetLabels for f64 {}
impl GetLabels for char {}
impl GetLabels for bool {}
impl GetLabels for () {}

pub trait GetStaticStrFromIntoStaticStr<'a> {
    fn __autometrics_static_str(&'a self) -> Option<&'static str>;
}

impl<'a, T: 'a> GetStaticStrFromIntoStaticStr<'a> for T
where
    &'static str: From<&'a T>,
{
    fn __autometrics_static_str(&'a self) -> Option<&'static str> {
        Some(self.into())
    }
}

pub trait GetStaticStr {
    fn __autometrics_static_str(&self) -> Option<&'static str> {
        None
    }
}

impl<T> GetStaticStr for &T {}

impl GetStaticStr for i8 {}
impl GetStaticStr for i16 {}
impl GetStaticStr for i32 {}
impl GetStaticStr for i64 {}
impl GetStaticStr for i128 {}
impl GetStaticStr for isize {}
impl GetStaticStr for u8 {}
impl GetStaticStr for u16 {}
impl GetStaticStr for u32 {}
impl GetStaticStr for u64 {}
impl GetStaticStr for u128 {}
impl GetStaticStr for usize {}
impl GetStaticStr for f32 {}
impl GetStaticStr for f64 {}
impl GetStaticStr for char {}
impl GetStaticStr for bool {}
impl GetStaticStr for () {}

// Copied from https://github.com/SkylerLipthay/case/blob/33ef733c5e6da8d0898ff17e125cd3b9c086bcf0/src/lib.rs#L144-L162
fn snake_case(string: &str) -> String {
    // The first character is never prepended with an underscore, so skip it even if it is an
    // uppercase ASCII character.
    let underscore_count = string
        .chars()
        .skip(1)
        .filter(|&c| c.is_ascii_uppercase())
        .count();
    let mut result = String::with_capacity(string.len() + underscore_count);

    for (i, c) in string.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}
