//! Connect metrics to traces using exemplars
//!
//! Exemplars are a newer Prometheus / OpenMetrics / OpenTelemetry feature that allows you to associate
//! specific traces or samples with a given metric. This enables you to investigate what caused metrics
//! to change by looking at individual examples that contributed to the metrics.
//!
//! Autometrics integrates with tracing libraries to extract details from the
//! current span context and attach them as exemplars to the metrics it generates.
//!
//! See each of the submodules for details on how to integrate with each tracing library.
//!
//! # Supported Metrics Libraries
//!
//! Exemplars are currently only supported with the `prometheus-client` metrics library,
//! because that is the only one that currently supports producing metrics with exemplars.
//!
//! # Exposing Metrics to Prometheus with Exemplars
//!
//! To enable Prometheus to scrape metrics with exemplars you must:
//! 1. Run Prometheus with the [`--enable-feature=exemplar-storage`](https://prometheus.io/docs/prometheus/latest/feature_flags/#exemplars-storage) flag
//! 2. Export the metrics to Prometheus using [`prometheus_exporter::encode_http_response`] or
//!   make sure to manually set the `Content-Type` header to indicate it is using the the OpenMetrics format:
//!   ```http
//!   Content-Type: application/openmetrics-text; version=1.0.0; charset=utf-8
//!   ```

use std::collections::HashMap;

#[cfg(feature = "exemplars-opentelemetry")]
mod opentelemetry;
#[cfg(feature = "exemplars-tracing")]
pub mod tracing;

#[cfg(all(feature = "exemplars-opentelemetry", feature = "exemplars-tracing"))]
compile_error!("Only one of the exemplars-opentelemetry and exemplars-tracing features can be enabled at a time");

#[cfg(not(any(feature = "exemplars-opentelemetry", feature = "exemplars-tracing")))]
compile_error!("One of the exemplars-opentelemetry or exemplars-tracing features must be enabled");

#[cfg(not(feature = "prometheus-client"))]
compile_error!("Exemplars can only be used with the `prometheus-client` metrics library because that is the only one that currently supports producing metrics with exemplars");

pub(crate) type TraceLabels = HashMap<&'static str, String>;
pub(crate) fn get_exemplar() -> Option<TraceLabels> {
    #[cfg(feature = "exemplars-opentelemetry")]
    {
        opentelemetry::get_exemplar()
    }
    #[cfg(feature = "exemplars-tracing")]
    {
        tracing::get_exemplar()
    }
}
