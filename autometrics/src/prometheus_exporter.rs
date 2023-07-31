//! Helper functions for easily collecting and exporting metrics to Prometheus.
//!
//! This module is compatible with any of the metrics backends. It uses
//! the `prometheus-client` by default if you do not specifically enable another backend.
//!
//! You do not need this module if you are already collecting custom metrics and exporting them to Prometheus.
//!
//! # Example
//! ```rust
//! use autometrics::prometheus_exporter::{self, PrometheusResponse};
//!
//! /// Exports metrics to Prometheus.
//! /// This should be mounted on `/metrics` on your API server
//! pub async fn get_metrics() -> PrometheusResponse {
//!     prometheus_exporter::encode_http_response()
//! }
//!
//! pub fn main() {
//!     prometheus_exporter::init();
//! }
//! ```

#[cfg(debug_assertions)]
use crate::__private::{AutometricsTracker, TrackMetrics, FUNCTION_DESCRIPTIONS};
use crate::settings::{get_settings, AutometricsSettings};
use http::{header::CONTENT_TYPE, Response};
#[cfg(metrics)]
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
#[cfg(opentelemetry)]
use opentelemetry_api::metrics::MetricsError;
#[cfg(any(opentelemetry, prometheus))]
use prometheus::TextEncoder;
use thiserror::Error;

#[cfg(not(exemplars))]
/// Prometheus text format content type
const RESPONSE_CONTENT_TYPE: &str = "text/plain; version=0.0.4";
#[cfg(exemplars)]
/// OpenMetrics content type
const RESPONSE_CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

static GLOBAL_EXPORTER: OnceCell<GlobalPrometheus> = OnceCell::new();

pub type PrometheusResponse = Response<String>;

#[derive(Debug, Error)]
pub enum EncodingError {
    #[cfg(any(prometheus, opentelemetry))]
    #[error(transparent)]
    Prometheus(#[from] prometheus::Error),

    #[cfg(prometheus_client)]
    #[error(transparent)]
    Format(#[from] std::fmt::Error),

    #[error(transparent)]
    Initialization(#[from] ExporterInitializationError),
}

#[derive(Debug, Error)]
pub enum ExporterInitializationError {
    #[error("Prometheus exporter has already been initialized")]
    AlreadyInitialized,

    #[cfg(opentelemetry)]
    #[error(transparent)]
    OpenTelemetryExporter(#[from] MetricsError),

    #[cfg(metrics)]
    #[error(transparent)]
    MetricsExporter(#[from] BuildError),
}

/// Initialize the global Prometheus metrics collector and exporter.
///
/// You will need a collector/exporter set up in order to use the metrics
/// generated by autometrics. You can either use this one or configure
/// your own using your metrics backend.
///
/// In debug builds, this will also set the function call counters to zero.
/// This exposes the names of instrumented functions to Prometheus without
/// affecting the metric values.
///
/// You should not call this function if you initialize the Autometrics
/// settings via [`AutometricsSettingsBuilder::try_init`].
///
/// [`AutometricsSettingsBuilder::try_init`]: crate::settings::AutometricsSettingsBuilder::try_init
pub fn try_init() -> Result<(), ExporterInitializationError> {
    // Initialize the global exporter but only if it hasn't already been initialized
    let mut newly_initialized = false;
    GLOBAL_EXPORTER.get_or_try_init(|| {
        newly_initialized = true;
        initialize_prometheus_exporter()
    })?;

    if !newly_initialized {
        return Err(ExporterInitializationError::AlreadyInitialized);
    }

    // Set all of the function counters to zero
    #[cfg(debug_assertions)]
    AutometricsTracker::intitialize_metrics(&FUNCTION_DESCRIPTIONS);

    Ok(())
}

/// Initialize the global Prometheus metrics collector and exporter.
///
/// You will need a collector/exporter set up in order to use the metrics
/// generated by autometrics. You can either use this one or configure
/// your own using your metrics backend.
///
/// This should be included in your `main.rs`:
/// ```
/// # fn main() {
/// # #[cfg(feature="prometheus-exporter")]
///     autometrics::prometheus_exporter::init();
/// # }
/// ```
///
/// In debug builds, this will also set the function call counters to zero.
/// This exposes the names of instrumented functions to Prometheus without
/// affecting the metric values.
///
/// You should not call this function if you initialize the Autometrics
/// settings via [`AutometricsSettingsBuilder::init`].
///
/// [`AutometricsSettingsBuilder::init`]: crate::settings::AutometricsSettingsBuilder::init
///
/// # Panics
///
/// Panics if the exporter has already been initialized.
pub fn init() {
    try_init().unwrap();
}

/// Export the collected metrics to the Prometheus format.
///
/// Create a handler on your API (often, this would be the
/// handler for the route `/metrics`) that returns the result of this function.
///
/// For example, using Axum, you might have a handler:
/// ```rust
/// # use http::StatusCode;
/// // Mounted at the route `/metrics`
/// pub async fn metrics_get() -> (StatusCode, String) {
///   match autometrics::prometheus_exporter::encode_to_string() {
///     Ok(metrics) => (StatusCode::OK, metrics),
///     Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
///   }
/// }
/// ```
pub fn encode_to_string() -> Result<String, EncodingError> {
    GLOBAL_EXPORTER
        .get_or_try_init(initialize_prometheus_exporter)?
        .encode_metrics()
}

/// Export the collected metrics to the Prometheus or OpenMetrics format and wrap
/// them in an HTTP response.
///
/// If you are using exemplars, this will automatically use the OpenMetrics
/// content type so that Prometheus can scrape the metrics and exemplars.
pub fn encode_http_response() -> PrometheusResponse {
    match encode_to_string() {
        Ok(metrics) => http::Response::builder()
            .status(200)
            .header(CONTENT_TYPE, RESPONSE_CONTENT_TYPE)
            .body(metrics)
            .expect("Error building response"),
        Err(err) => http::Response::builder()
            .status(500)
            .body(format!("{:?}", err))
            .expect("Error building response"),
    }
}

#[derive(Clone)]
#[doc(hidden)]
struct GlobalPrometheus {
    #[allow(dead_code)]
    settings: &'static AutometricsSettings,
    #[cfg(metrics)]
    metrics_exporter: PrometheusHandle,
}

impl GlobalPrometheus {
    fn encode_metrics(&self) -> Result<String, EncodingError> {
        let mut output = String::new();

        #[cfg(metrics)]
        output.push_str(&self.metrics_exporter.render());

        #[cfg(any(prometheus, opentelemetry))]
        TextEncoder::new().encode_utf8(&self.settings.prometheus_registry.gather(), &mut output)?;

        #[cfg(prometheus_client)]
        prometheus_client::encoding::text::encode(
            &mut output,
            &self.settings.prometheus_client_registry,
        )?;

        Ok(output)
    }
}

fn initialize_prometheus_exporter() -> Result<GlobalPrometheus, ExporterInitializationError> {
    let settings = get_settings();

    #[cfg(opentelemetry)]
    {
        use opentelemetry_api::global;
        use opentelemetry_prometheus::exporter;
        use opentelemetry_sdk::metrics::reader::AggregationSelector;
        use opentelemetry_sdk::metrics::{Aggregation, InstrumentKind, MeterProvider};

        /// A custom aggregation selector that uses the configured histogram buckets,
        /// along with the other default aggregation settings.
        struct AggregationSelectorWithHistogramBuckets {
            histogram_buckets: Vec<f64>,
        }

        impl AggregationSelector for AggregationSelectorWithHistogramBuckets {
            fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
                match kind {
                    InstrumentKind::Counter
                    | InstrumentKind::UpDownCounter
                    | InstrumentKind::ObservableCounter
                    | InstrumentKind::ObservableUpDownCounter => Aggregation::Sum,
                    InstrumentKind::ObservableGauge => Aggregation::LastValue,
                    InstrumentKind::Histogram => Aggregation::ExplicitBucketHistogram {
                        boundaries: self.histogram_buckets.clone(),
                        record_min_max: false,
                    },
                }
            }
        }

        let exporter = exporter()
            .with_registry(settings.prometheus_registry.clone())
            .with_aggregation_selector(AggregationSelectorWithHistogramBuckets {
                histogram_buckets: settings.histogram_buckets.clone(),
            })
            .without_scope_info()
            .without_target_info()
            .build()?;

        let meter_provider = MeterProvider::builder().with_reader(exporter).build();

        global::set_meter_provider(meter_provider);
    }

    Ok(GlobalPrometheus {
        #[cfg(metrics)]
        metrics_exporter: PrometheusBuilder::new()
            .set_buckets(&settings.histogram_buckets)?
            .install_recorder()?,
        settings,
    })
}
