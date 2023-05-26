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
//! 2. Set the `Content-Type` header on the response from the Prometheus metrics endpoint to indicate
//!    it is using the OpenMetrics exposition format instead of the default Prometheus format.
//!   ```http
//!   Content-Type: application/openmetrics-text; version=1.0.0; charset=utf-8
//!   ```
//!
//!  ```rust
//!   use http::{header::CONTENT_TYPE, Response};
//!
//!   /// Expose the metrics to Prometheus in the OpenMetrics format
//!   async fn get_metrics() -> Response<String> {
//!       match autometrics::encode_global_metrics() {
//!           Ok(metrics) => Response::builder()
//!               .header(
//!                   CONTENT_TYPE,
//!                   "application/openmetrics-text; version=1.0.0; charset=utf-8",
//!               )
//!               .body(metrics)
//!               .unwrap(),
//!           Err(err) => Response::builder()
//!               .status(500)
//!               .body(err.to_string())
//!               .unwrap(),
//!       }
//!   }
//!   ```

#[cfg(feature = "exemplars-tracing")]
pub mod tracing;
