//! Connect metrics to traces using exemplars.
//!
//! Exemplars are a newer Prometheus / OpenMetrics / OpenTelemetry feature that allows you to associate
//! specific traces or samples with a given metric. This enables you to investigate what caused metrics
//! to change by looking at individual examples that contributed to the metrics.
//!
//! Autometrics integrates with tracing libraries to extract details from the
//! current span context and automatically attach them as exemplars to the generated metrics.
//!
//! # Supported metrics libraries
//!
//! Exemplars are currently only supported with the `prometheus-client` metrics library,
//! because that is the only one that currently supports producing metrics with exemplars.
//!
//! # Exposing metrics to Prometheus with exemplars
//!
//! To enable Prometheus to scrape metrics with exemplars you must:
//! 1. Run Prometheus with the [`--enable-feature=exemplar-storage`](https://prometheus.io/docs/prometheus/latest/feature_flags/#exemplars-storage) flag
//! 2. Export the metrics to Prometheus using the provided [`prometheus_exporter::encode_http_response`] or
//!   make sure to manually set the `Content-Type` header to indicate it is using the the OpenMetrics format,
//!   rather than the default Prometheus format:
//!   ```http
//!   Content-Type: application/openmetrics-text; version=1.0.0; charset=utf-8
//!   ```
//!
//! [`prometheus_exporter::encode_http_response`]: crate::prometheus_exporter::encode_http_response
//!
//! # Tracing libraries
//!
//! ## [`tracing`](https://crates.io/crates/tracing)
//!
//! See the [`tracing` submodule docs](tracing).
//!
//! ## [`tracing-opentelemetry`](https://crates.io/crates/tracing-opentelemetry)
//!
//! Extract exemplars from the OpenTelemetry Context attached to the current tracing Span.
//!
//! This works in the following way:
//! 1. Add the [`tracing_opentelemetry::OpenTelemetryLayer`] to your tracing subscriber
//! 2. That layer ensures that there is an [`opentelemetry::Context`] attached to every [`tracing::Span`]
//! 3. Spans can be manually created or created for every function using the [`tracing::instrument`] macro
//! 4. Autometrics extracts the `trace_id` and `span_id` from the `Context` and attaches them as exemplars to the generated metrics
//!
//! ### Example
//!
//! ```rust
//! # use autometrics::autometrics;
//! use tracing_subscriber::prelude::*;
//! use tracing_opentelemetry::OpenTelemetryLayer;
//!
//! let tracer = opentelemetry_sdk::export::trace::stdout::new_pipeline().install_simple();
//! let otel_layer = OpenTelemetryLayer::new(tracer);
//!
//! // Create a tracing subscriber with the OpenTelemetry layer
//! tracing_subscriber::fmt()
//!   .finish()
//!   .with(otel_layer)
//!   .init();
//!
//! #[autometrics]
//! #[tracing::instrument]
//! async fn my_function() {
//!   // This function produces metrics with exemplars
//!   // that contain a trace_id and span_id
//! }
//! ```
//!
//! [`tracing_opentelemetry::OpenTelemetryLayer`]: https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/struct.OpenTelemetryLayer.html
//! [`opentelemetry::Context`]: https://docs.rs/opentelemetry/latest/opentelemetry/struct.Context.html
//! [`tracing::Span`]: https://docs.rs/tracing/latest/tracing/struct.Span.html
//! [`tracing::instrument`]: https://docs.rs/tracing/latest/tracing/attr.instrument.html

use std::collections::HashMap;

#[cfg(exemplars_tracing)]
pub mod tracing;
#[cfg(exemplars_tracing_opentelemetry)]
mod tracing_opentelemetry;

#[cfg(all(not(doc), exemplars_tracing, exemplars_tracing_opentelemetry))]
compile_error!("Only one of the exemplars-tracing and exemplars-tracing-opentelemetry features can be enabled at a time");

#[cfg(not(prometheus_client))]
compile_error!("Exemplars can only be used with the `prometheus-client` metrics library because that is the only one that currently supports producing metrics with exemplars");

pub(crate) type TraceLabels = HashMap<&'static str, String>;
pub(crate) fn get_exemplar() -> Option<TraceLabels> {
    #[cfg(exemplars_tracing_opentelemetry)]
    {
        tracing_opentelemetry::get_exemplar()
    }
    #[cfg(exemplars_tracing)]
    {
        tracing::get_exemplar()
    }
}
