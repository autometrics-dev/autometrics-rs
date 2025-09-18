use opentelemetry_otlp::{
    ExportConfig, ExporterBuildError, MetricExporter, Protocol, WithExportConfig,
    OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use std::ops::Deref;
use std::time::Duration;

/// Newtype struct holding a [`SdkMeterProvider`] with a custom `Drop` implementation to automatically clean up itself
#[repr(transparent)]
#[must_use = "Assign this to a unused variable instead: `let _meter = ...` (NOT `let _ = ...`), as else it will be dropped immediately - which will cause it to be shut down"]
pub struct OtelMeterProvider(SdkMeterProvider);

impl Deref for OtelMeterProvider {
    type Target = SdkMeterProvider;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for OtelMeterProvider {
    fn drop(&mut self) {
        // this will only error if `.shutdown` gets called multiple times
        let _ = self.0.shutdown();
    }
}

/// Initialize the OpenTelemetry push exporter using HTTP transport.
///
/// # Interval and timeout
/// This function uses the environment variables `OTEL_METRIC_EXPORT_TIMEOUT` and `OTEL_METRIC_EXPORT_INTERVAL`
/// to configure the timeout and interval respectively. If you want to customize those
/// from within code, consider using [`init_http_with_timeout_period`].
#[cfg(feature = "otel-push-exporter-http")]
pub fn init_http(url: impl Into<String>) -> Result<OtelMeterProvider, ExporterBuildError> {
    let (timeout, period) = timeout_and_period_from_env_or_default();
    init_http_with_timeout_period(url, timeout, period)
}

/// Initialize the OpenTelemetry push exporter using HTTP transport with customized `timeout` and `period`.
#[cfg(feature = "otel-push-exporter-http")]
pub fn init_http_with_timeout_period(
    url: impl Into<String>,
    timeout: Duration,
    period: Duration,
) -> Result<OtelMeterProvider, ExporterBuildError> {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_export_config(ExportConfig {
            endpoint: Some(url.into()),
            protocol: Protocol::HttpBinary,
            timeout: Some(timeout),
            ..Default::default()
        })
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(period)
        .build();

    Ok(OtelMeterProvider(runtime().with_reader(reader).build()))
}

/// Initialize the OpenTelemetry push exporter using gRPC transport.
///
/// # Interval and timeout
/// This function uses the environment variables `OTEL_METRIC_EXPORT_TIMEOUT` and `OTEL_METRIC_EXPORT_INTERVAL`
/// to configure the timeout and interval respectively. If you want to customize those
/// from within code, consider using [`init_grpc_with_timeout_period`].
#[cfg(feature = "otel-push-exporter-grpc")]
pub fn init_grpc(url: impl Into<String>) -> Result<OtelMeterProvider, ExporterBuildError> {
    let (timeout, period) = timeout_and_period_from_env_or_default();
    init_grpc_with_timeout_period(url, timeout, period)
}

/// Initialize the OpenTelemetry push exporter using gRPC transport with customized `timeout` and `period`.
#[cfg(feature = "otel-push-exporter-grpc")]
pub fn init_grpc_with_timeout_period(
    url: impl Into<String>,
    timeout: Duration,
    period: Duration,
) -> Result<OtelMeterProvider, ExporterBuildError> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_export_config(ExportConfig {
            endpoint: Some(url.into()),
            protocol: Protocol::HttpBinary,
            timeout: Some(timeout),
            ..Default::default()
        })
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(period)
        .build();

    Ok(OtelMeterProvider(runtime().with_reader(reader).build()))
}

/// returns timeout and period from their respective environment variables
/// or the default, if they are not set or set to an invalid value
fn timeout_and_period_from_env_or_default() -> (Duration, Duration) {
    const OTEL_EXPORTER_TIMEOUT_ENV: &str = "OTEL_METRIC_EXPORT_TIMEOUT";
    const OTEL_EXPORTER_INTERVAL_ENV: &str = "OTEL_METRIC_EXPORT_INTERVAL";

    let timeout = Duration::from_secs(
        std::env::var_os(OTEL_EXPORTER_TIMEOUT_ENV)
            .and_then(|os_string| os_string.into_string().ok())
            .and_then(|str| str.parse().ok())
            .unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT.as_secs()),
    );

    let period = Duration::from_secs(
        std::env::var_os(OTEL_EXPORTER_INTERVAL_ENV)
            .and_then(|os_string| os_string.into_string().ok())
            .and_then(|str| str.parse().ok())
            .unwrap_or(60),
    );

    (timeout, period)
}

fn runtime() -> MeterProviderBuilder {
    SdkMeterProvider::builder()
}
