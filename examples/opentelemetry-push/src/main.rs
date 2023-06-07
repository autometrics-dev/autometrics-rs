use autometrics::autometrics;
use opentelemetry::{runtime, Context};
use opentelemetry::sdk::export::metrics::aggregation::cumulative_temporality_selector;
use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry::sdk::metrics::selectors;
use opentelemetry::metrics;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use tokio::time::sleep;
use std::error::Error;
use std::time::Duration;

fn init_metrics() -> metrics::Result<BasicController> {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..ExportConfig::default()
    };
    opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            cumulative_temporality_selector(),
            runtime::Tokio,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .build()
}

#[autometrics]
fn do_stuff() {
    println!("Doing stuff...");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let meter_provider = init_metrics()?;
    let cx = Context::current();

    for _ in 0..10 {
        do_stuff();
    }

    println!("Waiting so that we could see metrics being pushed via OTLP every 10 seconds...");
    sleep(Duration::from_secs(60)).await;
    meter_provider.stop(&cx)?;

    Ok(())
}
