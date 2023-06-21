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
pub const FUNCTION_KEY: &str = "function";
pub const MODULE_KEY: &str = "module";
pub const CALLER_KEY: &str = "caller";
pub const RESULT_KEY: &str = "result";
pub const OK_KEY: &str = "ok";
pub const ERROR_KEY: &str = "error";
pub const OBJECTIVE_NAME: &str = "objective.name";
pub const OBJECTIVE_NAME_PROMETHEUS: &str = "objective_name";
pub const OBJECTIVE_PERCENTILE: &str = "objective.percentile";
pub const OBJECTIVE_PERCENTILE_PROMETHEUS: &str = "objective_percentile";
pub const OBJECTIVE_LATENCY_THRESHOLD: &str = "objective.latency.threshold";
pub const OBJECTIVE_LATENCY_THRESHOLD_PROMETHEUS: &str = "objective_latency_threshold";
pub const VERSION_KEY: &str = "version";
pub const COMMIT_KEY: &str = "commit";
pub const BRANCH_KEY: &str = "branch";
pub const SERVICE_NAME_KEY: &str = "service.name";
pub const SERVICE_NAME_KEY_PROMETHEUS: &str = "service_name";
