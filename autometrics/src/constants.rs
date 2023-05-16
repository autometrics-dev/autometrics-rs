// Metrics
pub const COUNTER_NAME: &str = "function.calls.count";
pub const HISTOGRAM_NAME: &str = "function.calls.duration";
pub const GAUGE_NAME: &str = "function.calls.concurrent";
pub const BUILD_INFO_NAME: &str = "build_info";

// Prometheus-flavored metric names
pub const COUNTER_NAME_PROMETHEUS: &str = "function_calls_count";
pub const HISTOGRAM_NAME_PROMETHEUS: &str = "function_calls_duration";
pub const GAUGE_NAME_PROMETHEUS: &str = "function_calls_concurrent";

// Descriptions
pub const COUNTER_DESCRIPTION: &str = "Autometrics counter for tracking function calls";
pub const HISTOGRAM_DESCRIPTION: &str = "Autometrics histogram for tracking function call duration";
pub const GAUGE_DESCRIPTION: &str = "Autometrics gauge for tracking concurrent function calls";
pub const BUILD_INFO_DESCRIPTION: &str =
    "Autometrics info metric for tracking software version and build details";

// Labels
pub const FUNCTION_KEY: &'static str = "function";
pub const MODULE_KEY: &'static str = "module";
pub const CALLER_KEY: &'static str = "caller";
pub const RESULT_KEY: &'static str = "result";
pub const OK_KEY: &'static str = "ok";
pub const ERROR_KEY: &'static str = "error";
pub const OBJECTIVE_NAME: &'static str = "objective.name";
pub const OBJECTIVE_NAME_PROMETHEUS: &'static str = "objective_name";
pub const OBJECTIVE_PERCENTILE: &'static str = "objective.percentile";
pub const OBJECTIVE_PERCENTILE_PROMETHEUS: &'static str = "objective_percentile";
pub const OBJECTIVE_LATENCY_THRESHOLD: &'static str = "objective.latency.threshold";
pub const OBJECTIVE_LATENCY_THRESHOLD_PROMETHEUS: &'static str = "objective_latency_threshold";
pub const VERSION_KEY: &'static str = "version";
pub const COMMIT_KEY: &'static str = "commit";
pub const BRANCH_KEY: &'static str = "branch";
