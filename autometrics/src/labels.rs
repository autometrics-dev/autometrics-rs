use crate::{constants::*, objectives::*, settings::get_settings};
#[cfg(prometheus_client)]
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue, LabelValueEncoder};

pub(crate) type Label = (&'static str, &'static str);
pub type ResultAndReturnTypeLabels = (&'static str, Option<&'static str>);

/// These are the labels used for the `build_info` metric.
#[cfg_attr(
    prometheus_client,
    derive(EncodeLabelSet, Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct BuildInfoLabels {
    pub(crate) branch: &'static str,
    pub(crate) commit: &'static str,
    pub(crate) version: &'static str,
    pub(crate) service_name: &'static str,
    pub(crate) repo_url: &'static str,
    pub(crate) repo_provider: &'static str,
    pub(crate) autometrics_version: &'static str,
}

impl BuildInfoLabels {
    pub fn new(version: &'static str, commit: &'static str, branch: &'static str, repo_url: &'static str, mut repo_provider: &'static str) -> Self {
        if repo_provider.is_empty() {
            repo_provider = Self::determinate_repo_provider_from_url(repo_url);
        }

        Self {
            version,
            commit,
            branch,
            service_name: &get_settings().service_name,
            repo_url,
            repo_provider,
            autometrics_version: "1.0.0"
        }
    }

    pub fn to_vec(&self) -> Vec<Label> {
        vec![
            (COMMIT_KEY, self.commit),
            (VERSION_KEY, self.version),
            (BRANCH_KEY, self.branch),
            (SERVICE_NAME_KEY, self.service_name),
            (REPO_URL_KEY, self.repo_url),
            (REPO_PROVIDER_KEY, self.repo_provider),
            (AUTOMETRICS_VERSION_KEY, self.autometrics_version)
        ]
    }

    fn determinate_repo_provider_from_url(url: &'static str) -> &'static str {
        let lowered = url.to_lowercase();

        if lowered.contains("github.com") {
            "github"
        } else if lowered.contains("gitlab.com") {
            "gitlab"
        } else if lowered.contains("bitbucket.org") {
            "bitbucket"
        } else {
            ""
        }
    }
}

/// These are the labels used for the `function.calls` metric.
#[cfg_attr(
    prometheus_client,
    derive(EncodeLabelSet, Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct CounterLabels {
    pub(crate) function: &'static str,
    pub(crate) module: &'static str,
    pub(crate) service_name: &'static str,
    pub(crate) caller_function: &'static str,
    pub(crate) caller_module: &'static str,
    pub(crate) result: Option<ResultLabel>,
    pub(crate) ok: Option<&'static str>,
    pub(crate) error: Option<&'static str>,
    pub(crate) objective_name: Option<&'static str>,
    pub(crate) objective_percentile: Option<ObjectivePercentile>,
}

#[cfg_attr(prometheus_client, derive(Debug, Clone, PartialEq, Eq, Hash))]
pub(crate) enum ResultLabel {
    Ok,
    Error,
}

impl ResultLabel {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ResultLabel::Ok => OK_KEY,
            ResultLabel::Error => ERROR_KEY,
        }
    }
}

#[cfg(prometheus_client)]
impl EncodeLabelValue for ResultLabel {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), std::fmt::Error> {
        match self {
            ResultLabel::Ok => EncodeLabelValue::encode(&OK_KEY, encoder),
            ResultLabel::Error => EncodeLabelValue::encode(&ERROR_KEY, encoder),
        }
    }
}

impl CounterLabels {
    pub fn new(
        function: &'static str,
        module: &'static str,
        caller_function: &'static str,
        caller_module: &'static str,
        result: Option<ResultAndReturnTypeLabels>,
        objective: Option<Objective>,
    ) -> Self {
        let (objective_name, objective_percentile) = if let Some(objective) = objective {
            if let Some(success_rate) = objective.success_rate {
                (Some(objective.name), Some(success_rate))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        let (result, ok, error) = if let Some((result, return_value_type)) = result {
            match result {
                OK_KEY => (Some(ResultLabel::Ok), return_value_type, None),
                ERROR_KEY => (Some(ResultLabel::Error), None, return_value_type),
                _ => (None, None, None),
            }
        } else {
            (None, None, None)
        };
        Self {
            function,
            module,
            service_name: &get_settings().service_name,
            caller_function,
            caller_module,
            objective_name,
            objective_percentile,
            result,
            ok,
            error,
        }
    }

    pub fn to_vec(&self) -> Vec<Label> {
        let mut labels = vec![
            (FUNCTION_KEY, self.function),
            (MODULE_KEY, self.module),
            (SERVICE_NAME_KEY, self.service_name),
            (CALLER_FUNCTION_KEY, self.caller_function),
            (CALLER_MODULE_KEY, self.caller_module),
        ];
        if let Some(result) = &self.result {
            labels.push((RESULT_KEY, result.as_str()));
        }
        if let Some(ok) = self.ok {
            labels.push((OK_KEY, ok));
        }
        if let Some(error) = self.error {
            labels.push((ERROR_KEY, error));
        }
        if let Some(objective_name) = self.objective_name {
            labels.push((OBJECTIVE_NAME, objective_name));
        }
        if let Some(objective_percentile) = &self.objective_percentile {
            labels.push((OBJECTIVE_PERCENTILE, objective_percentile.as_str()));
        }

        labels
    }
}

/// These are the labels used for the `function.calls.duration` metric.
#[cfg_attr(
    prometheus_client,
    derive(EncodeLabelSet, Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct HistogramLabels {
    pub(crate) function: &'static str,
    pub(crate) module: &'static str,
    pub(crate) service_name: &'static str,
    pub(crate) objective_name: Option<&'static str>,
    pub(crate) objective_percentile: Option<ObjectivePercentile>,
    pub(crate) objective_latency_threshold: Option<ObjectiveLatency>,
}

impl HistogramLabels {
    pub fn new(function: &'static str, module: &'static str, objective: Option<Objective>) -> Self {
        let (objective_name, objective_percentile, objective_latency_threshold) =
            if let Some(objective) = objective {
                if let Some((latency, percentile)) = objective.latency {
                    (Some(objective.name), Some(percentile), Some(latency))
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

        Self {
            function,
            module,
            service_name: &get_settings().service_name,
            objective_name,
            objective_percentile,
            objective_latency_threshold,
        }
    }

    pub fn to_vec(&self) -> Vec<Label> {
        let mut labels = vec![
            (FUNCTION_KEY, self.function),
            (MODULE_KEY, self.module),
            (SERVICE_NAME_KEY, self.service_name),
        ];

        if let Some(objective_name) = self.objective_name {
            labels.push((OBJECTIVE_NAME, objective_name));
        }
        if let Some(objective_percentile) = &self.objective_percentile {
            labels.push((OBJECTIVE_PERCENTILE, objective_percentile.as_str()));
        }
        if let Some(objective_latency_threshold) = &self.objective_latency_threshold {
            labels.push((
                OBJECTIVE_LATENCY_THRESHOLD,
                objective_latency_threshold.as_str(),
            ));
        }

        labels
    }
}

/// These are the labels used for the `function.calls.concurrent` metric.
#[cfg_attr(
    prometheus_client,
    derive(EncodeLabelSet, Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct GaugeLabels {
    pub(crate) function: &'static str,
    pub(crate) module: &'static str,
    pub(crate) service_name: &'static str,
}

impl GaugeLabels {
    pub fn new(function: &'static str, module: &'static str) -> Self {
        Self {
            function,
            module,
            service_name: &get_settings().service_name,
        }
    }

    pub fn to_array(&self) -> Vec<Label> {
        vec![
            (FUNCTION_KEY, self.function),
            (MODULE_KEY, self.module),
            (SERVICE_NAME_KEY, self.service_name),
        ]
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

/// A trait to override the inferred label for the "result" of a function call.
pub trait GetLabels {
    fn __autometrics_get_labels(&self) -> Option<&'static str>;
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
impl_trait_for_types!(GetStaticStr);

/// Return the value of labels to use for the "result" counter according to
/// the value's exact type and attributes.
///
/// The macro uses the autoref specialization trick through spez to get the labels for the type in a variety of circumstances.
/// Specifically, if the value is a Result, it will add the ok or error label accordingly unless one or both of the types that
/// the Result<T, E> is generic over implements the GetLabels trait. The label allows to override the inferred label, and the
/// [`ResultLabels`](crate::ResultLabels) macro implements the GetLabels trait for the user using annotations.
///
/// The macro is meant to be called with a reference as argument: `get_result_labels_for_value(&return_value)`
///
/// See: <https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md>
#[doc(hidden)]
#[macro_export]
macro_rules! get_result_labels_for_value {
    ($e:expr) => {{
        use $crate::__private::{
            GetLabels, GetStaticStr, ResultAndReturnTypeLabels, ERROR_KEY, OK_KEY,
        };
        $crate::__private::spez! {
            for val = $e;

            match<T, E> &::std::result::Result<T, E> where T: GetLabels, E: GetLabels -> ::std::option::Option<ResultAndReturnTypeLabels> {
                match val {
                    Ok(ok) => Some((
                        ok.__autometrics_get_labels().unwrap_or(OK_KEY),
                        ok.__autometrics_static_str(),
                    )),
                    Err(err) => Some((
                        err.__autometrics_get_labels().unwrap_or(ERROR_KEY),
                        err.__autometrics_static_str(),
                    )),
                }
            }

            match<T, E> &::std::result::Result<T, E> where E: GetLabels -> ::std::option::Option<ResultAndReturnTypeLabels> {
                match val {
                    Ok(ok) => Some((
                        OK_KEY,
                        ok.__autometrics_static_str(),
                    )),
                    Err(err) => Some((
                        err.__autometrics_get_labels().unwrap_or(ERROR_KEY),
                        err.__autometrics_static_str(),
                    )),
                }
            }

            match<T, E> &::std::result::Result<T, E> where T: GetLabels -> ::std::option::Option<ResultAndReturnTypeLabels> {
                match val {
                    Ok(ok) => Some((
                        ok.__autometrics_get_labels().unwrap_or(OK_KEY),
                        ok.__autometrics_static_str(),
                    )),
                    Err(err) => Some((
                        ERROR_KEY,
                        err.__autometrics_static_str(),
                    )),
                }
            }

            match<T, E> &::std::result::Result<T, E> -> ::std::option::Option<ResultAndReturnTypeLabels> {
                match val {
                    Ok(ok) => Some((
                        OK_KEY,
                        ok.__autometrics_static_str(),
                    )),
                    Err(err) => Some((
                        ERROR_KEY,
                        err.__autometrics_static_str(),
                    )),
                }
            }

            match<T> &T where T: GetLabels -> ::std::option::Option<ResultAndReturnTypeLabels> {
                val.__autometrics_get_labels().map(|label| (label, val.__autometrics_static_str()))
            }

            match<T> T -> ::std::option::Option<ResultAndReturnTypeLabels> {
                None
            }
        }
    }};
}
