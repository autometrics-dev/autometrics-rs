//! # Create alerts from your function-level metrics using Service-Level Objective (SLO) best practices.
//!
//! Autometrics makes it easy to add Prometheus alerts using Service-Level Objectives (SLOs) to a function or group of functions.
//!
//! This works using pre-defined [Prometheus alerting rules](https://github.com/autometrics-dev/autometrics-shared#prometheus-recording--alerting-rules), which can be loaded via the `rule_files` field in your Prometheus configuration. By default, most of the recording rules are dormant. They are enabled by specific metric labels that can be automatically attached by autometrics.
//!
//! To use autometrics SLOs and alerts, create one or multiple [`Objective`]s based on the function(s) success rate and/or latency, as shown below. The `Objective` can be passed as an argument to the `autometrics` macro to include the given function in that objective.
//!
//! Once you've added objectives to your code, you can use the [Autometrics Service-Level Objectives(SLO) Dashboard](https://github.com/autometrics-dev/autometrics-shared#dashboards) to visualize the current status of your objective(s).
//!
//! ## Example
//!
//! ```rust
//! use autometrics::autometrics;
//! use autometrics::objectives::{Objective, ObjectiveLatency, ObjectivePercentile};
//!
//! const API_SLO: Objective = Objective::new("api")
//!     .success_rate(ObjectivePercentile::P99_9)
//!     .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P99);
//!
//! #[autometrics(objective = API_SLO)]
//! pub fn api_handler() {
//!   // ...
//! }
//! ```

#[cfg(prometheus_client)]
use prometheus_client::encoding::{EncodeLabelValue, LabelValueEncoder};

/// A Service-Level Objective (SLO) for a function or group of functions.
///
/// The objective should be given a descriptive name and can represent
/// a success rate and/or latency objective.
///
/// For details on SLOs, see [The Google SRE Book](https://sre.google/sre-book/service-level-objectives/).
///
/// ## Example
/// ```rust
/// # use autometrics::{autometrics, objectives::*};
/// const API_SLO: Objective = Objective::new("api")
///     .success_rate(ObjectivePercentile::P99_9)
///     .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P99);
///
/// #[autometrics(objective = API_SLO)]
/// pub fn api_handler() {
///    // ...
/// }
/// ```
///
/// ## How this works
///
/// When an objective is added to a function, Autometrics attaches additional labels to the generated metrics.
///
/// Specifically, [`success_rate`] objectives will add objective-related labels to the `function.calls` metric
/// and [`latency`] objectives will add labels to the `function.calls.duration` metric.
///
/// Autometrics comes with a set of Prometheus [recording rules](https://prometheus.io/docs/prometheus/latest/configuration/recording_rules/)
/// and [alerting rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
/// that will fire alerts when the given objective is being violated.
///
/// By default, these recording rules will effectively lay dormaint.
/// However, they are enabled when the special labels are present on certain metrics.
///
/// [`success_rate`]: Objective::success_rate
/// [`latency`]: Objective::latency
pub struct Objective {
    pub(crate) name: &'static str,
    pub(crate) success_rate: Option<ObjectivePercentile>,
    pub(crate) latency: Option<(ObjectiveLatency, ObjectivePercentile)>,
}

impl Objective {
    /// Create a new objective with the given name.
    ///
    /// The name should be something descriptive of the function or group of functions it covers.
    /// For example, if you have an objective covering all of the HTTP handlers in your API you might call it `"api"`.
    pub fn new(name: &'static str) -> Self {
        if !name.chars().all(char::is_alphanumeric) {
            eprintln!("warning: objective name \"{name}\" should be alphanumeric");
        }

        Objective {
            name,
            success_rate: None,
            latency: None,
        }
    }

    /// Specify the success rate for this objective.
    ///
    /// This means that the function or group of functions that are part of this objective
    /// should return an `Ok` result at least this percentage of the time.
    ///
    /// ## Metric Labels
    ///
    /// When a success rate objective is added to a function, the `function.calls` metric
    /// will have these labels added:
    /// - `objective.name` - the value of the name passed to the [`Objective::new`] function
    /// - `objective.percentile` - the percentile provided here
    pub const fn success_rate(mut self, percentile: ObjectivePercentile) -> Self {
        self.success_rate = Some(percentile);
        self
    }

    /// Specify the latency and percentile for this objective.
    ///
    /// This means that the function or group of functions that are part of this objective
    /// should complete in less than the given latency at least this percentage of the time.
    ///
    /// ## Metric Labels
    ///
    /// When a latency objective is added to a function, the `function.calls.duration` metric,
    /// which will appear in Prometheus as `function_calls_duration_bucket`, `function_calls_duration_count`,
    /// and `function_calls_duration_sum`, will have these labels added:
    /// - `objective.name` - the value of the name passed to the [`Objective::new`] function
    /// - `objective.latency_threshold` - the latency threshold provided here
    /// - `objective.percentile` - the percentile provided here
    pub const fn latency(
        mut self,
        latency_threshold: ObjectiveLatency,
        percentile: ObjectivePercentile,
    ) -> Self {
        self.latency = Some((latency_threshold, percentile));
        self
    }
}

/// The percentage of requests that must meet the given criteria (success rate or latency).
#[cfg_attr(any(prometheus_client, debug_assertions), derive(Clone, Copy))]
#[cfg_attr(prometheus_client, derive(Debug, PartialEq, Eq, Hash))]
#[non_exhaustive]
pub enum ObjectivePercentile {
    /// 90%
    P90,
    /// 95%
    P95,
    /// 99%
    P99,
    /// 99.9%
    P99_9,
    /// ⚠️ **Careful when using this option!**
    ///
    /// In order for this to work with the recording and alerting rules, you need to:
    /// 1. generate a custom Sloth file using the [autometrics-cli](https://github.com/autometrics-dev/autometrics-rs/tree/main/autometrics-cli) that includes this objective
    /// 2. use [Sloth](https://sloth.dev) to generate the Prometheus recording and alerting rules
    /// 3. configure your Prometheus instance to use the generated rules
    #[cfg(feature = "custom-objective-percentile")]
    Custom(&'static str),
}

impl ObjectivePercentile {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ObjectivePercentile::P90 => "90",
            ObjectivePercentile::P95 => "95",
            ObjectivePercentile::P99 => "99",
            ObjectivePercentile::P99_9 => "99.9",
            #[cfg(feature = "custom-objective-percentile")]
            ObjectivePercentile::Custom(custom) => custom,
        }
    }
}

#[cfg(prometheus_client)]
impl EncodeLabelValue for ObjectivePercentile {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), std::fmt::Error> {
        self.as_str().encode(encoder)
    }
}

/// The latency threshold, in milliseoncds, for a given objective.
#[cfg_attr(prometheus_client, derive(Clone, Debug, PartialEq, Eq, Hash))]
#[non_exhaustive]
pub enum ObjectiveLatency {
    /// 5 milliseconds
    Ms5,
    /// 10 milliseconds
    Ms10,
    /// 25 milliseconds
    Ms25,
    /// 50 milliseconds
    Ms50,
    /// 75 milliseconds
    Ms75,
    /// 100 milliseconds
    Ms100,
    /// 150 milliseconds
    Ms250,
    /// 500 milliseconds
    Ms500,
    /// 750 milliseconds
    Ms750,
    /// 1 second
    Ms1000,
    /// 2.5 seconds
    Ms2500,
    /// 5 seconds
    Ms5000,
    /// 7.5 seconds
    Ms7500,
    /// 10 seconds
    Ms10000,
    /// ⚠️ **Careful when using this option!**
    ///
    /// First, the latency should be specified in seconds, not milliseconds.
    /// For example, if you want to specify a latency of 200 milliseconds,
    /// you would specify `ObjectiveLatency::Custom("0.2")`.
    ///
    /// Second, you must ensure that this value matches
    /// one of the histogram buckets configured in the
    /// [Autometrics settings](crate::settings::AutometricsSettingsBuilder::histogram_buckets)
    /// or in your metrics exporter.
    /// If it is not, the alerting rules will not work.
    /// This is because the queries and recording rules compare this to
    /// the value of the `le` label on the histogram buckets.
    #[cfg(feature = "custom-objective-latency")]
    Custom(&'static str),
}

impl ObjectiveLatency {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ObjectiveLatency::Ms5 => "0.005",
            ObjectiveLatency::Ms10 => "0.01",
            ObjectiveLatency::Ms25 => "0.025",
            ObjectiveLatency::Ms50 => "0.05",
            ObjectiveLatency::Ms75 => "0.075",
            ObjectiveLatency::Ms100 => "0.1",
            ObjectiveLatency::Ms250 => "0.25",
            ObjectiveLatency::Ms500 => "0.5",
            ObjectiveLatency::Ms750 => "0.75",
            ObjectiveLatency::Ms1000 => "1",
            ObjectiveLatency::Ms2500 => "2.5",
            ObjectiveLatency::Ms5000 => "5",
            ObjectiveLatency::Ms7500 => "7.5",
            ObjectiveLatency::Ms10000 => "10",
            #[cfg(feature = "custom-objective-latency")]
            ObjectiveLatency::Custom(custom) => custom,
        }
    }
}

#[cfg(prometheus_client)]
impl EncodeLabelValue for ObjectiveLatency {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), std::fmt::Error> {
        self.as_str().encode(encoder)
    }
}
