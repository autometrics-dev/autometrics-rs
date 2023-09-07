use opentelemetry_api::metrics::MetricsError;
use opentelemetry_otlp::OtlpMetricPipeline;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::runtime;
use std::ops::Deref;
use std::time::Duration;

#[repr(transparent)]
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
        .with_period(Duration::from_secs(1))
        .build()
        .map(OtelMeterProvider)
}

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
        .with_period(Duration::from_secs(1))
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
