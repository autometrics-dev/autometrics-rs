use crate::{constants::*, objectives::*};
use std::ops::Deref;

pub(crate) type Label = (&'static str, &'static str);
type ResultAndReturnTypeLabels = (&'static str, Option<&'static str>);

/// These are the labels used for the `function.calls.count` metric.
pub struct CounterLabels {
    pub(crate) function: &'static str,
    pub(crate) module: &'static str,
    pub(crate) caller: &'static str,
    pub(crate) result: Option<ResultAndReturnTypeLabels>,
    pub(crate) objective: Option<(&'static str, ObjectivePercentile)>,
}

impl CounterLabels {
    pub fn new(
        function: &'static str,
        module: &'static str,
        caller: &'static str,
        result: Option<ResultAndReturnTypeLabels>,
        objective: Option<Objective>,
    ) -> Self {
        let objective = if let Some(objective) = objective {
            if let Some(success_rate) = objective.success_rate {
                Some((objective.name, success_rate))
            } else {
                None
            }
        } else {
            None
        };
        Self {
            function,
            module,
            caller,
            result,
            objective,
        }
    }

    pub fn to_vec(&self) -> Vec<Label> {
        let mut labels = vec![
            (FUNCTION_KEY, self.function),
            (MODULE_KEY, self.module),
            (CALLER_KEY, self.caller),
        ];
        if let Some((result, return_value_type)) = self.result {
            labels.push((RESULT_KEY, result));
            if let Some(return_value_type) = return_value_type {
                labels.push((result, return_value_type));
            }
        }
        if let Some((name, percentile)) = &self.objective {
            labels.push((OBJECTIVE_NAME, name));
            labels.push((OBJECTIVE_PERCENTILE, percentile.as_str()));
        }

        labels
    }
}

/// These are the labels used for the `function.calls.duration` metric.
pub struct HistogramLabels {
    pub function: &'static str,
    pub module: &'static str,
    /// The SLO name, objective percentile, and latency threshold
    pub objective: Option<(&'static str, ObjectivePercentile, ObjectiveLatency)>,
}

impl HistogramLabels {
    pub fn new(function: &'static str, module: &'static str, objective: Option<Objective>) -> Self {
        let objective = if let Some(objective) = objective {
            if let Some((latency, percentile)) = objective.latency {
                Some((objective.name, percentile, latency))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            function,
            module,
            objective,
        }
    }

    pub fn to_vec(&self) -> Vec<Label> {
        let mut labels = vec![(FUNCTION_KEY, self.function), (MODULE_KEY, self.module)];

        if let Some((name, percentile, latency)) = &self.objective {
            labels.push((OBJECTIVE_NAME, name));
            labels.push((OBJECTIVE_PERCENTILE, percentile.as_str()));
            labels.push((OBJECTIVE_LATENCY_THRESHOLD, latency.as_str()));
        }

        labels
    }
}

/// These are the labels used for the `function.calls.concurrent` metric.
pub struct GaugeLabels {
    pub function: &'static str,
    pub module: &'static str,
}

impl GaugeLabels {
    pub fn to_array(&self) -> [Label; 2] {
        [(FUNCTION_KEY, self.function), (MODULE_KEY, self.module)]
    }
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
    fn __autometrics_get_labels(&self) -> Option<ResultAndReturnTypeLabels> {
        None
    }
}

impl<T, E> GetLabelsFromResult for Result<T, E> {
    fn __autometrics_get_labels(&self) -> Option<ResultAndReturnTypeLabels> {
        self.get_label().map(|(k, v)| (k, Some(v)))
    }
}

pub enum LabelArray {
    Three([Label; 3]),
    Four([Label; 4]),
    Five([Label; 5]),
}

impl Deref for LabelArray {
    type Target = [Label];

    fn deref(&self) -> &Self::Target {
        match self {
            LabelArray::Three(l) => l,
            LabelArray::Four(l) => l,
            LabelArray::Five(l) => l,
        }
    }
}

pub trait GetLabels {
    fn __autometrics_get_labels(&self) -> Option<ResultAndReturnTypeLabels> {
        None
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

pub trait GetLabel {
    fn get_label(&self) -> Option<(&'static str, &'static str)> {
        None
    }
}
impl_trait_for_types!(GetLabel);

#[cfg(test)]
mod tests {
    use super::*;
    use autometrics_macros::GetLabel;

    #[test]
    fn custom_trait_implementation() {
        struct CustomResult;

        impl GetLabel for CustomResult {
            fn get_label(&self) -> Option<(&'static str, &'static str)> {
                Some(("ok", "my-result"))
            }
        }

        assert_eq!(Some(("ok", "my-result")), CustomResult {}.get_label());
    }

    #[test]
    fn manual_enum() {
        enum MyFoo {
            A,
            B,
        }

        impl GetLabel for MyFoo {
            fn get_label(&self) -> Option<(&'static str, &'static str)> {
                Some(("hello", match self {
                    MyFoo::A => "a",
                    MyFoo::B => "b",
                }))
            }
        }

        assert_eq!(Some(("hello", "a")), MyFoo::A.get_label());
        assert_eq!(Some(("hello", "b")), MyFoo::B.get_label());
    }

    #[test]
    fn derived_enum() {
        #[derive(GetLabel)]
        #[autometrics(label_key = "my_foo")]
        enum MyFoo {
            #[autometrics(label_value = "hello")]
            Alpha,
            #[autometrics()]
            BetaValue,
            Charlie,
        }

        assert_eq!(Some(("my_foo", "hello")), MyFoo::Alpha.get_label());
        assert_eq!(Some(("my_foo", "beta_value")), MyFoo::BetaValue.get_label());
        assert_eq!(Some(("my_foo", "charlie")), MyFoo::Charlie.get_label());
    }
}
