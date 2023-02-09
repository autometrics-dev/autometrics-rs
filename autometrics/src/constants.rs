// Metrics
pub(crate) const COUNTER_NAME: &str = "function.calls.count";
pub(crate) const HISTOGRAM_NAME: &str = "function.calls.duration";
pub(crate) const GAUGE_NAME: &str = "function.calls.concurrent";

// Descriptions
pub(crate) const COUNTER_DESCRIPTION: &str = "Autometrics counter for tracking function calls";
pub(crate) const HISTOGRAM_DESCRIPTION: &str =
    "Autometrics histogram for tracking function call duration";
pub(crate) const GAUGE_DESCRIPTION: &str =
    "Autometrics gauge for tracking concurrent function calls";

// Labels
pub(crate) const FUNCTION_KEY: &'static str = "function";
pub(crate) const MODULE_KEY: &'static str = "module";
pub(crate) const CALLER_KEY: &'static str = "caller";
pub(crate) const RESULT_KEY: &'static str = "result";
pub(crate) const OK_KEY: &'static str = "ok";
pub(crate) const ERROR_KEY: &'static str = "error";
