#[cfg(feature = "metrics")]
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::Lazy;
use opentelemetry_prometheus::{exporter, PrometheusExporter};
use opentelemetry_sdk::export::metrics::aggregation;
use opentelemetry_sdk::metrics::{controllers, processors, selectors};
use prometheus::{default_registry, Error, TextEncoder};

const HISTOGRAM_BUCKETS: [f64; 10] = [0.01, 0.025, 0.05, 0.075, 0.1, 0.15, 0.2, 0.35, 0.5, 1.0];
static GLOBAL_EXPORTER: Lazy<GlobalPrometheus> = Lazy::new(|| initialize_metrics_exporter());

#[derive(Clone)]
#[doc(hidden)]
pub struct GlobalPrometheus {
    exporter: PrometheusExporter,
    #[cfg(feature = "metrics")]
    handle: PrometheusHandle,
}

impl GlobalPrometheus {
    fn encode_metrics(&self) -> Result<String, Error> {
        let metric_families = self.exporter.registry().gather();
        let encoder = TextEncoder::new();
        #[allow(unused_mut)]
        let mut output = encoder.encode_to_string(&metric_families)?;

        #[cfg(feature = "metrics")]
        {
            output.push('\n');
            output.push_str(&self.handle.render());
        }

        Ok(output)
    }
}

/// Initialize the global Prometheus metrics collector and exporter.
///
/// You will need a collector/exporter set up in order to use the metrics
/// generated by autometrics. You can either use this one or configure
/// your own following the example from the
/// [`opentelemetry_prometheus`](https://docs.rs/opentelemetry-prometheus/latest/opentelemetry_prometheus/)
/// crate documentation.
///
/// This should be included in your `main.rs`:
/// ```rust
/// # main() {
/// let _exporter = global_metrics_exporter();
/// # }
/// ```
pub fn global_metrics_exporter() -> GlobalPrometheus {
    GLOBAL_EXPORTER.clone()
}

/// Prometheus needs a metrics endpoint to scrape metrics from.
///
/// Create a handler on your API (often, this would be the
/// handler for the route `/metrics`) that returns the result of this function.
///
/// For example, using Axum, you might have a handler:
/// ```rust
/// // Mounted at the route `/metrics`
/// pub fn metrics_get() -> (StatusCode, String) {
///   match autometrics::encode_global_metrics() {
///     Ok(metrics) => (StatusCode::OK, metrics),
///     Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
///   }
/// }
/// ```
pub fn encode_global_metrics() -> Result<String, Error> {
    GLOBAL_EXPORTER.encode_metrics()
}

fn initialize_metrics_exporter() -> GlobalPrometheus {
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram(HISTOGRAM_BUCKETS),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    // Use the prometheus crate's default registry so it still works with custom
    // metrics defined through the prometheus crate
    let registry = default_registry().clone();
    let prometheus_exporter = exporter(controller).with_registry(registry).init();

    GlobalPrometheus {
        exporter: prometheus_exporter,

        #[cfg(feature = "metrics")]
        handle: PrometheusBuilder::new()
            .set_buckets(&HISTOGRAM_BUCKETS)
            .expect("Failed to set histogram buckets")
            .install_recorder()
            .expect("Failed to install recorder"),
    }
}