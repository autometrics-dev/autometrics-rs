//! Customize the global settings for Autometrics.
//!
//! See [`AutometricsSettingsBuilder`] for more details on the available options.

use once_cell::sync::OnceCell;
use thiserror::Error;

pub(crate) static AUTOMETRICS_SETTINGS: OnceCell<AutometricsSettings> = OnceCell::new();

/// Load the settings configured by the user or use the defaults.
///
/// Note that attempting to set the settings after this function is called will panic.
#[allow(dead_code)]
pub(crate) fn get_settings() -> &'static AutometricsSettings {
    AUTOMETRICS_SETTINGS.get_or_init(|| AutometricsSettings::default())
}

#[derive(Debug)]
pub(crate) struct AutometricsSettings {
    /// The buckets used for the function latency histograms.
    ///
    /// By default, we use the buckets recommended by the [OpenTelemetry specification].
    ///
    /// [OpenTelemetry specification]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#explicit-bucket-histogram-aggregation
    #[allow(dead_code)]
    pub(crate) histogram_buckets: Vec<f64>,
}

impl Default for AutometricsSettings {
    fn default() -> Self {
        Self {
            histogram_buckets: vec![
                0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
            ],
        }
    }
}

/// Customize the global settings for Autometrics.
pub struct AutometricsSettingsBuilder {
    histogram_buckets: Vec<f64>,
}

impl AutometricsSettingsBuilder {
    pub fn new() -> Self {
        Self {
            histogram_buckets: vec![],
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

    /// Set the global settings for Autometrics. This returns an error if the
    /// settings have already been initialized.
    ///
    /// Note: this function should only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    pub fn try_init(self) -> Result<(), AlreadyInitializedError> {
        let settings = AutometricsSettings {
            histogram_buckets: self.histogram_buckets,
        };
        AUTOMETRICS_SETTINGS
            .set(settings)
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
