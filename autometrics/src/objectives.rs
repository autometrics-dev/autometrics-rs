/// This represents a Service-Level Objective (SLO) for a function or group of functions.
/// The objective should be given a descriptive name and can represent
/// a success rate and/or latency objective.
///
/// For details on SLOs, see <https://sre.google/sre-book/service-level-objectives/>
///
/// Example:
/// ```rust
/// # use autometrics::{autometrics, objectives::*};
/// const API_SLO: Objective = Objective::new("api")
///     .success_rate(ObjectivePercentile::P99_9)
///     .latency(TargetLatency::Ms200, ObjectivePercentile::P99);
///
/// #[autometrics(objective = API_SLO)]
/// pub fn api_handler() {
///    // ...
/// }
/// ```
///
/// ## How this works
///
/// When an objective is added to a function, the metrics for that function will
/// have additional labels attached to specify the SLO details.
///
/// Autometrics comes with a set of Prometheus [recording rules](https://prometheus.io/docs/prometheus/latest/configuration/recording_rules/)
/// and [alerting rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
/// that will fire alerts when the given objective is being violated.
///
/// By default, these recording rules will effectively lay dormaint.
/// However, they are enabled when the special labels are present on certain metrics.
pub struct Objective {
    pub(crate) name: &'static str,
    pub(crate) success_rate: Option<ObjectivePercentile>,
    pub(crate) latency: Option<(ObjectiveLatency, ObjectivePercentile)>,
}

impl Objective {
    /// Create a new objective with the given name.
    ///
    /// The name should be something descriptive of the function or group of functions it covers.
    /// For example, if you have an objective covering all of the HTTP handlers in your API you might call it "api".
    pub const fn new(name: &'static str) -> Self {
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
    pub const fn success_rate(mut self, success_rate: ObjectivePercentile) -> Self {
        self.success_rate = Some(success_rate);
        self
    }

    /// Specify the latency and percentile for this objective.
    ///
    /// This means that the function or group of functions that are part of this objective
    /// should complete in less than the given latency at least this percentage of the time.
    pub const fn latency(
        mut self,
        target_latency: ObjectiveLatency,
        percentile: ObjectivePercentile,
    ) -> Self {
        self.latency = Some((target_latency, percentile));
        self
    }
}

/// The percentage of requests that must meet the given criteria (success rate or latency).
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
    /// ?????? Careful when using this option!
    ///
    /// In order for this to work with the recording and alerting rules, you need to:
    /// 1. generate a custom Sloth file using the autometrics-cli that includes this objective
    /// 2. use Sloth to generate the Prometheus recording and alerting rules
    /// 3. configure your Prometheus instance to use the generated rules
    #[cfg(feature = "custom-objectives")]
    Custom(&'static str),
}

impl ObjectivePercentile {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ObjectivePercentile::P90 => "90",
            ObjectivePercentile::P95 => "95",
            ObjectivePercentile::P99 => "99",
            ObjectivePercentile::P99_9 => "99.9",
            #[cfg(feature = "custom-objectives")]
            ObjectivePercentile::Custom(custom) => custom,
        }
    }
}

/// The latency threshold, in milliseoncds, for a given objective.
#[non_exhaustive]
pub enum ObjectiveLatency {
    /// 10ms
    Ms10,
    /// 25ms
    Ms25,
    /// 50ms
    Ms50,
    /// 75ms
    Ms75,
    /// 100ms
    Ms100,
    /// 150ms
    Ms150,
    /// 200ms
    Ms200,
    /// 350ms
    Ms350,
    /// 500ms
    Ms500,
    /// 1000ms
    Ms1000,
    /// ?????? Careful when using this option!
    ///
    /// First, the latency should be specified in seconds, not milliseconds.
    ///
    /// Second, you must ensure that this value matches
    /// one of the histogram buckets configured in your metrics exporter.
    /// If it is not, the alerting rules will not work.
    /// This is because the recording rules compare this to the value
    /// of the `le` label on the histogram buckets.
    #[cfg(feature = "custom-objectives")]
    Custom(&'static str),
}

impl ObjectiveLatency {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            ObjectiveLatency::Ms10 => "0.01",
            ObjectiveLatency::Ms25 => "0.025",
            ObjectiveLatency::Ms50 => "0.05",
            ObjectiveLatency::Ms75 => "0.075",
            ObjectiveLatency::Ms100 => "0.1",
            ObjectiveLatency::Ms150 => "0.15",
            ObjectiveLatency::Ms200 => "0.2",
            ObjectiveLatency::Ms350 => "0.35",
            ObjectiveLatency::Ms500 => "0.5",
            ObjectiveLatency::Ms1000 => "1",
            #[cfg(feature = "custom-objectives")]
            ObjectiveLatency::Custom(custom) => custom,
        }
    }
}
