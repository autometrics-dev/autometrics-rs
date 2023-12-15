use opentelemetry::metrics::MetricsError;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_otlp::{OtlpMetricPipeline, OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT};
use opentelemetry_sdk::metrics::MeterProvider;
use std::ops::Deref;
use std::time::Duration;

/// Newtype struct holding a [`MeterProvider`] with a custom `Drop` implementation to automatically clean up itself
#[repr(transparent)]
#[must_use = "Assign this to a unused variable instead: `let _meter = ...` (NOT `let _ = ...`), as else it will be dropped immediately - which will cause it to be shut down"]
pub struct OtelMeterProvider(MeterProvider);

impl Deref for OtelMeterProvider {
    type Target = MeterProvider;

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
pub fn init_http(url: impl Into<String>) -> Result<OtelMeterProvider, MetricsError> {
    let (timeout, period) = timeout_and_period_from_env_or_default();
    init_http_with_timeout_period(url, timeout, period)
}

/// Initialize the OpenTelemetry push exporter using HTTP transport with customized `timeout` and `period`.
#[cfg(feature = "otel-push-exporter-http")]
pub fn init_http_with_timeout_period(
    url: impl Into<String>,
    timeout: Duration,
    period: Duration,
) -> Result<OtelMeterProvider, MetricsError> {
    runtime()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_export_config(ExportConfig {
                    endpoint: url.into(),
                    protocol: Protocol::HttpBinary,
                    timeout,
                    ..Default::default()
                }),
        )
        .with_period(period)
        .build()
        .map(OtelMeterProvider)
}

/// Initialize the OpenTelemetry push exporter using gRPC transport.
///
/// # Interval and timeout
/// This function uses the environment variables `OTEL_METRIC_EXPORT_TIMEOUT` and `OTEL_METRIC_EXPORT_INTERVAL`
/// to configure the timeout and interval respectively. If you want to customize those
/// from within code, consider using [`init_grpc_with_timeout_period`].
#[cfg(feature = "otel-push-exporter-grpc")]
pub fn init_grpc(url: impl Into<String>) -> Result<OtelMeterProvider, MetricsError> {
    let (timeout, period) = timeout_and_period_from_env_or_default();
    init_grpc_with_timeout_period(url, timeout, period)
}

/// Initialize the OpenTelemetry push exporter using gRPC transport with customized `timeout` and `period`.
#[cfg(feature = "otel-push-exporter-grpc")]
pub fn init_grpc_with_timeout_period(
    url: impl Into<String>,
    timeout: Duration,
    period: Duration,
) -> Result<OtelMeterProvider, MetricsError> {
    runtime()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(ExportConfig {
                    endpoint: url.into(),
                    protocol: Protocol::Grpc,
                    timeout,
                    ..Default::default()
                }),
        )
        .with_period(period)
        .build()
        .map(OtelMeterProvider)
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
            .unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
    );

    let period = Duration::from_secs(
        std::env::var_os(OTEL_EXPORTER_INTERVAL_ENV)
            .and_then(|os_string| os_string.into_string().ok())
            .and_then(|str| str.parse().ok())
            .unwrap_or(60),
    );

    (timeout, period)
}

#[cfg(all(
    feature = "otel-push-exporter-tokio",
    not(any(
        feature = "otel-push-exporter-tokio-current-thread",
        feature = "otel-push-exporter-async-std"
    ))
))]
fn runtime(
) -> OtlpMetricPipeline<opentelemetry_sdk::runtime::Tokio, opentelemetry_otlp::NoExporterConfig> {
    return opentelemetry_otlp::new_pipeline().metrics(opentelemetry_sdk::runtime::Tokio);
}

#[cfg(all(
    feature = "otel-push-exporter-tokio-current-thread",
    not(any(
        feature = "otel-push-exporter-tokio",
        feature = "otel-push-exporter-async-std"
    ))
))]
fn runtime() -> OtlpMetricPipeline<
    opentelemetry_sdk::runtime::TokioCurrentThread,
    opentelemetry_otlp::NoExporterConfig,
> {
    return opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::TokioCurrentThread);
}

#[cfg(all(
    feature = "otel-push-exporter-async-std",
    not(any(
        feature = "otel-push-exporter-tokio",
        feature = "otel-push-exporter-tokio-current-thread"
    ))
))]
fn runtime(
) -> OtlpMetricPipeline<opentelemetry_sdk::runtime::AsyncStd, opentelemetry_otlp::NoExporterConfig>
{
    return opentelemetry_otlp::new_pipeline().metrics(opentelemetry_sdk::runtime::AsyncStd);
}

#[cfg(not(any(
    feature = "otel-push-exporter-tokio",
    feature = "otel-push-exporter-tokio-current-thread",
    feature = "otel-push-exporter-async-std"
)))]
fn runtime() -> ! {
    compile_error!("select your runtime (`otel-push-exporter-tokio`, `otel-push-exporter-tokio-current-thread` or `otel-push-exporter-async-std`) for the autometrics push exporter or use the custom push exporter if none fit")
}
