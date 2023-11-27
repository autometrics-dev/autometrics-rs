use opentelemetry::metrics::MetricsError;
use opentelemetry_otlp::OtlpMetricPipeline;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
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
    runtime()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_export_config(ExportConfig {
                    endpoint: url.into(),
                    protocol: Protocol::HttpBinary,
                    ..Default::default()
                }),
        )
        .build()
        .map(OtelMeterProvider)
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
    runtime()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(ExportConfig {
                    endpoint: url.into(),
                    protocol: Protocol::Grpc,
                    ..Default::default()
                }),
        )
        .build()
        .map(OtelMeterProvider)
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
