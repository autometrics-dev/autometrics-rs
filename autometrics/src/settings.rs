//! Customize the global settings for Autometrics.
//!
//! See [`AutometricsSettings`] for more details on the available options.

use once_cell::sync::OnceCell;
use std::env;
use thiserror::Error;

pub(crate) static AUTOMETRICS_SETTINGS: OnceCell<AutometricsSettings> = OnceCell::new();
const DEFAULT_HISTOGRAM_BUCKETS: [f64; 14] = [
    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

/// Load the settings configured by the user or use the defaults.
///
/// Note that attempting to set the settings after this function is called will panic.
#[allow(dead_code)]
pub(crate) fn get_settings() -> &'static AutometricsSettings {
    AUTOMETRICS_SETTINGS.get_or_init(|| AutometricsSettings::default())
}

#[derive(Debug)]
pub struct AutometricsSettings {
    #[allow(dead_code)]
    pub(crate) histogram_buckets: Vec<f64>,
    pub(crate) service_name: String,
}

impl Default for AutometricsSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl AutometricsSettings {
    pub fn new() -> Self {
        Self {
            histogram_buckets: DEFAULT_HISTOGRAM_BUCKETS.to_vec(),
            service_name: env::var("AUTOMETRICS_SERVICE_NAME")
                .or_else(|_| env::var("OTEL_SERVICE_NAME"))
                .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
        }
    }

    /// Set the buckets, represented in seconds, used for the function latency histograms.
    ///
    /// If this is not set, the buckets recommended by the [OpenTelemetry specification] are used.
    ///
    /// [OpenTelemetry specification]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#explicit-bucket-histogram-aggregation
    pub fn histogram_buckets(mut self, histogram_buckets: impl Into<Vec<f64>>) -> Self {
        self.histogram_buckets = histogram_buckets.into();
        self
    }

    /// The name of the service. This is mostly useful when the same
    /// code base is used for multiple services, so that it is easy
    /// to distinguish the metrics produced by each instance.
    ///
    /// If this is not set programmatically, it will fall back first
    /// to the `AUTOMETRICS_SERVICE_NAME` environment variable,
    /// then `OTEL_SERVICE_NAME`, and finally the name of the crate
    /// as defined in the `Cargo.toml` file.
    pub fn service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set the global settings for Autometrics. This returns an error if the
    /// settings have already been initialized.
    ///
    /// Note: this function should only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    pub fn try_init(self) -> Result<(), AlreadyInitializedError> {
        AUTOMETRICS_SETTINGS
            .set(self)
            .map_err(|_| AlreadyInitializedError)
    }

    /// Set the global settings for Autometrics.
    ///
    /// Note: this function can only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    ///
    /// ## Panics
    ///
    /// This function will panic if the settings have already been initialized.
    pub fn init(self) {
        self.try_init().unwrap();
    }
}

#[derive(Debug, Error)]
#[error("Autometrics settings have already been initialized")]
pub struct AlreadyInitializedError;
