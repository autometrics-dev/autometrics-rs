// Metrics
pub const COUNTER_NAME: &str = "function.calls.count";
pub const HISTOGRAM_NAME: &str = "function.calls.duration";
pub const GAUGE_NAME: &str = "function.calls.concurrent";

// Descriptions
pub const COUNTER_DESCRIPTION: &str = "Autometrics counter for tracking function calls";
pub const HISTOGRAM_DESCRIPTION: &str = "Autometrics histogram for tracking function call duration";
pub const GAUGE_DESCRIPTION: &str = "Autometrics gauge for tracking concurrent function calls";

// Labels
pub const FUNCTION_KEY: &'static str = "function";
pub const MODULE_KEY: &'static str = "module";
pub const CALLER_KEY: &'static str = "caller";
pub const RESULT_KEY: &'static str = "result";
pub const OK_KEY: &'static str = "ok";
pub const ERROR_KEY: &'static str = "error";
pub const SLO_NAME: &'static str = "slo_name";
pub const OBJECTIVE: &'static str = "objective";
pub const TARGET_LATENCY: &'static str = "target_latency";
