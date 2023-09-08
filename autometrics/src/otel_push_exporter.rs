use opentelemetry_api::metrics::MetricsError;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_otlp::{OtlpMetricPipeline, OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT};
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::runtime;
use std::ops::Deref;
use std::time::Duration;

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

#[cfg(feature = "otel-push-exporter-http")]
pub fn init_http(url: impl Into<String>) -> Result<OtelMeterProvider, MetricsError> {
    init_http_with_timeout_period(
        url,
        Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
        Duration::from_secs(1),
    )
}

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

#[cfg(feature = "otel-push-exporter-grpc")]
pub fn init_grpc(url: impl Into<String>) -> Result<OtelMeterProvider, MetricsError> {
    init_grpc_with_timeout_period(
        url,
        Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
        Duration::from_secs(1),
    )
}

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
fn runtime() -> OtlpMetricPipeline<opentelemetry_sdk::runtime::Tokio> {
    return opentelemetry_otlp::new_pipeline().metrics(opentelemetry_sdk::runtime::Tokio);
}

#[cfg(all(
    feature = "otel-push-exporter-tokio-current-thread",
    not(any(
        feature = "otel-push-exporter-tokio",
        feature = "otel-push-exporter-async-std"
    ))
))]
fn runtime() -> OtlpMetricPipeline<opentelemetry_sdk::runtime::TokioCurrentThread> {
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
fn runtime() -> OtlpMetricPipeline<opentelemetry_sdk::runtime::AsyncStd> {
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
