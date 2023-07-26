//! Customize the global settings for Autometrics.
//!
//! See [`AutometricsSettings`] for more details on the available options.

#[cfg(prometheus_exporter)]
use crate::prometheus_exporter::{self, ExporterInitializationError};
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
    #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
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
            #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
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
    #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
    pub fn histogram_buckets(mut self, histogram_buckets: impl Into<Vec<f64>>) -> Self {
        self.histogram_buckets = histogram_buckets.into();
        self
    }

    /// All metrics produced by Autometrics have a label called `service.name`
    /// (or `service_name` when exported to Prometheus) attached to
    /// identify the logical service they are part of.
    ///
    /// You can set this here or via environment variables.
    ///
    /// The priority for where the service name is loaded from is:
    /// 1. This method
    /// 2. `AUTOMETRICS_SERVICE_NAME` (at runtime)
    /// 3. `OTEL_SERVICE_NAME` (at runtime)
    /// 4. `CARGO_PKG_NAME` (at compile time), which is the name of the crate defined in the `Cargo.toml` file.
    pub fn service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set the global settings for Autometrics. This returns an error if the
    /// settings have already been initialized.
    ///
    /// Note: this function should only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    ///
    /// If the Prometheus exporter is enabled, this will also initialize it.
    pub fn try_init(self) -> Result<(), SettingsInitializationError> {
        AUTOMETRICS_SETTINGS
            .set(self)
            .map_err(|_| SettingsInitializationError::AlreadyInitialized)?;

        #[cfg(prometheus_exporter)]
        prometheus_exporter::try_init()?;

        Ok(())
    }

    /// Set the global settings for Autometrics.
    ///
    /// Note: this function can only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    ///
    /// If the Prometheus exporter is enabled, this will also initialize it.
    ///
    /// ## Panics
    ///
    /// This function will panic if the settings have already been initialized.
    pub fn init(self) {
        self.try_init().unwrap();
    }
}

#[derive(Debug, Error)]
pub enum SettingsInitializationError {
    #[error("Autometrics settings have already been initialized")]
    AlreadyInitialized,

    #[cfg(prometheus_exporter)]
    #[error(transparent)]
    PrometheusExporter(#[from] ExporterInitializationError),
}
