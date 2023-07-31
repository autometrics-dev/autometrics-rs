//! Customize the global settings for Autometrics.
//!
//! ```rust
//! use autometrics::settings::AutometricsSettings;
//!
//! AutometricsSettings::builder()
//!    .service_name("test_service")
//!   .init();
//! ```
//!
//! See [`AutometricsSettingsBuilder`] for more details on the available options.

#[cfg(prometheus_exporter)]
use crate::prometheus_exporter::{self, ExporterInitializationError};
use once_cell::sync::OnceCell;
use std::env;
use thiserror::Error;

pub(crate) static AUTOMETRICS_SETTINGS: OnceCell<AutometricsSettings> = OnceCell::new();
#[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
const DEFAULT_HISTOGRAM_BUCKETS: [f64; 14] = [
    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

/// Load the settings configured by the user or use the defaults.
///
/// Note that attempting to set the settings after this function is called will panic.
#[allow(dead_code)]
pub(crate) fn get_settings() -> &'static AutometricsSettings {
    AUTOMETRICS_SETTINGS.get_or_init(|| AutometricsSettingsBuilder::default().build())
}

pub struct AutometricsSettings {
    #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
    pub(crate) histogram_buckets: Vec<f64>,
    pub(crate) service_name: String,
    #[cfg(any(prometheus, opentelemetry))]
    pub(crate) prometheus_registry: prometheus::Registry,
    #[cfg(prometheus_client)]
    pub(crate) prometheus_client_registry: prometheus_client::registry::Registry,
    #[cfg(prometheus_client)]
    pub(crate) prometheus_client_metrics: crate::tracker::prometheus_client::Metrics,
}

impl AutometricsSettings {
    pub fn builder() -> AutometricsSettingsBuilder {
        AutometricsSettingsBuilder::default()
    }

    /// Access the [`Registry`] where Autometrics metrics are collected.
    ///
    /// You can use this to encode the metrics using the functionality provided by the [`prometheus`] crate
    /// if you do not want to use the provided [`prometheus_exporter`].
    ///
    /// [`Registry`]: prometheus::Registry
    /// [`prometheus_exporter`]: crate::prometheus_exporter
    #[cfg(any(prometheus, opentelemetry))]
    pub fn prometheus_registry(&self) -> &prometheus::Registry {
        &self.prometheus_registry
    }

    /// Access the [`Registry`] where Autometrics metrics are collected.
    ///
    /// You can use this to encode the metrics using the functionality provided by the [`prometheus_client`] crate
    /// if you do not want to use the provided [`prometheus_exporter`].
    ///
    /// [`Registry`]: prometheus_client::registry::Registry
    /// [`prometheus_exporter`]: crate::prometheus_exporter
    #[cfg(prometheus_client)]
    pub fn prometheus_client_registry(&self) -> &prometheus_client::registry::Registry {
        &self.prometheus_client_registry
    }
}

#[derive(Debug, Default)]
pub struct AutometricsSettingsBuilder {
    pub(crate) service_name: Option<String>,
    #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
    pub(crate) histogram_buckets: Option<Vec<f64>>,
    #[cfg(any(prometheus, opentelemetry))]
    pub(crate) prometheus_registry: Option<prometheus::Registry>,
    #[cfg(prometheus_client)]
    pub(crate) prometheus_client_registry: Option<prometheus_client::registry::Registry>,
}

impl AutometricsSettingsBuilder {
    /// Set the buckets, represented in seconds, used for the function latency histograms.
    ///
    /// If this is not set, the buckets recommended by the [OpenTelemetry specification] are used.
    ///
    /// [OpenTelemetry specification]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk.md#explicit-bucket-histogram-aggregation
    #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
    pub fn histogram_buckets(mut self, histogram_buckets: impl Into<Vec<f64>>) -> Self {
        self.histogram_buckets = Some(histogram_buckets.into());
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
        self.service_name = Some(service_name.into());
        self
    }

    /// Configure the [`prometheus::Registry`] that will be used to collect metrics when using
    /// either the `prometheus` or `opentelemetry` backends. If none is set, it will use
    /// the [`prometheus::default_registry`].
    ///
    /// This is mainly useful if you want to add custom metrics to the same registry, or if you want to
    /// add a custom prefix or custom labels to all of the metrics.
    ///
    /// If you are not using the provided [`prometheus_exporter`] to export metrics and want to encode
    /// the metrics from the `Registry`, you can simply `clone` the `Registry` before passing it in here
    /// and use the original one for encoding.
    #[cfg(any(prometheus, opentelemetry))]
    pub fn prometheus_registry(mut self, registry: prometheus::Registry) -> Self {
        self.prometheus_registry = Some(registry);
        self
    }

    /// Configure the [`prometheus_client::registry::Registry`] that will be used to collect metrics.
    ///
    /// This is mainly useful if you want to add custom metrics to the same registry.
    ///
    /// If you are not using the provided [`prometheus_exporter`] to export metrics and want to access
    /// the `Registry` again to encode the metrics, you can access it again via [`AutometricsSettings::prometheus_client_registry`].
    #[cfg(prometheus_client)]
    pub fn prometheus_client_registry(
        mut self,
        registry: prometheus_client::registry::Registry,
    ) -> Self {
        self.prometheus_client_registry = Some(registry);
        self
    }

    /// Set the global settings for Autometrics. This returns an error if the
    /// settings have already been initialized.
    ///
    /// Note: this function should only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    ///
    /// If the Prometheus exporter is enabled, this will also initialize it.
    pub fn try_init(self) -> Result<&'static AutometricsSettings, SettingsInitializationError> {
        let settings = self.build();

        let settings = AUTOMETRICS_SETTINGS
            .try_insert(settings)
            .map_err(|_| SettingsInitializationError::AlreadyInitialized)?;

        #[cfg(prometheus_exporter)]
        prometheus_exporter::try_init()?;

        Ok(settings)
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
    pub fn init(self) -> &'static AutometricsSettings {
        self.try_init().unwrap()
    }

    fn build(self) -> AutometricsSettings {
        #[cfg(prometheus_client)]
        let (prometheus_client_registry, prometheus_client_metrics) =
            crate::tracker::prometheus_client::initialize_registry(
                self.prometheus_client_registry
                    .unwrap_or_else(|| <prometheus_client::registry::Registry>::default()),
            );

        AutometricsSettings {
            #[cfg(any(prometheus_exporter, prometheus, prometheus_client))]
            histogram_buckets: self
                .histogram_buckets
                .unwrap_or_else(|| DEFAULT_HISTOGRAM_BUCKETS.to_vec()),
            service_name: self
                .service_name
                .or_else(|| env::var("AUTOMETRICS_SERVICE_NAME").ok())
                .or_else(|| env::var("OTEL_SERVICE_NAME").ok())
                .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string()),
            #[cfg(prometheus_client)]
            prometheus_client_registry,
            #[cfg(prometheus_client)]
            prometheus_client_metrics,
            #[cfg(any(prometheus, opentelemetry))]
            prometheus_registry: self
                .prometheus_registry
                .unwrap_or_else(|| prometheus::default_registry().clone()),
        }
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
