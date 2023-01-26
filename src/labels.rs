use opentelemetry_api::{Key, KeyValue, Value};
use std::ops::Deref;

const FUNCTION_KEY: Key = Key::from_static_str("function");
const MODULE_KEY: Key = Key::from_static_str("module");
const CALLER_KEY: Key = Key::from_static_str("caller");
const RESULT_KEY: Key = Key::from_static_str("result");

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
    fn __autometrics_get_labels(
        &self,
        function: &'static str,
        module: &'static str,
        caller: &'static str,
    ) -> LabelArray;
}

impl<T, E> GetLabelsFromResult for Result<T, E> {
    fn __autometrics_get_labels(
        &self,
        function: &'static str,
        module: &'static str,
        caller: &'static str,
    ) -> LabelArray {
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
        let caller_label = KeyValue {
            key: CALLER_KEY,
            value: Value::String(caller.into()),
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
                value: Value::String(value.into()),
            };
            LabelArray::Five([
                function_label,
                module_label,
                caller_label,
                result_label,
                value_label,
            ])
        } else {
            LabelArray::Four([function_label, module_label, caller_label, result_label])
        }
    }
}

pub enum LabelArray {
    Four([KeyValue; 4]),
    Five([KeyValue; 5]),
}

impl Deref for LabelArray {
    type Target = [KeyValue];

    fn deref(&self) -> &Self::Target {
        match self {
            LabelArray::Four(l) => l,
            LabelArray::Five(l) => l,
        }
    }
}

pub trait GetLabels {
    fn __autometrics_get_labels(
        &self,
        function: &'static str,
        module: &'static str,
        _caller: &'static str,
    ) -> [KeyValue; 2] {
        create_labels(function, module)
    }
}

/// Implement the given trait for &T and all primitive types.
macro_rules! impl_trait_for_types {
    ($trait:ident) => {
        impl<T> $trait for &T {}
        impl $trait for i8 {}
        impl $trait for i16 {}
        impl $trait for i32 {}
        impl $trait for i64 {}
        impl $trait for i128 {}
        impl $trait for isize {}
        impl $trait for u8 {}
        impl $trait for u16 {}
        impl $trait for u32 {}
        impl $trait for u64 {}
        impl $trait for u128 {}
        impl $trait for usize {}
        impl $trait for f32 {}
        impl $trait for f64 {}
        impl $trait for char {}
        impl $trait for bool {}
        impl $trait for str {}
        impl $trait for () {}
        impl<A> $trait for (A,) {}
        impl<A, B> $trait for (A, B) {}
        impl<A, B, C> $trait for (A, B, C) {}
        impl<A, B, C, D> $trait for (A, B, C, D) {}
        impl<A, B, C, D, E> $trait for (A, B, C, D, E) {}
        impl<A, B, C, D, E, F> $trait for (A, B, C, D, E, F) {}
        impl<A, B, C, D, E, F, G> $trait for (A, B, C, D, E, F, G) {}
        impl<A, B, C, D, E, F, G, H> $trait for (A, B, C, D, E, F, G, H) {}
        impl<A, B, C, D, E, F, G, H, I> $trait for (A, B, C, D, E, F, G, H, I) {}
        impl<A, B, C, D, E, F, G, H, I, J> $trait for (A, B, C, D, E, F, G, H, I, J) {}
        impl<A, B, C, D, E, F, G, H, I, J, K> $trait for (A, B, C, D, E, F, G, H, I, J, K) {}
        impl<A, B, C, D, E, F, G, H, I, J, K, L> $trait for (A, B, C, D, E, F, G, H, I, J, K, L) {}
    };
}

impl_trait_for_types!(GetLabels);

trait GetStaticStrFromIntoStaticStr<'a> {
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

trait GetStaticStr {
    fn __autometrics_static_str(&self) -> Option<&'static str> {
        None
    }
}
impl_trait_for_types!(GetStaticStr);

pub(crate) fn create_labels(function_name: &'static str, module: &'static str) -> [KeyValue; 2] {
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
