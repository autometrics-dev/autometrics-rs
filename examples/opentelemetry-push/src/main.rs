use autometrics::autometrics;
use autometrics_example_util::sleep_random_duration;
use opentelemetry::{runtime, Context};
use opentelemetry::sdk::export::metrics::aggregation::cumulative_temporality_selector;
use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry::sdk::metrics::selectors;
use opentelemetry::metrics;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

fn init_metrics() -> metrics::Result<BasicController> {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..ExportConfig::default()
    };
    let push_interval = Duration::from_secs(1);
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
        .with_period(push_interval)
        .build()
}

#[autometrics]
async fn do_stuff() {
    println!("Doing stuff...");
    sleep_random_duration().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let meter_provider = init_metrics()?;
    let cx = Context::current();

    for _ in 0..100 {
        do_stuff().await;
    }

    println!("Waiting so that we could see metrics going down...");
    sleep(Duration::from_secs(10)).await;
    meter_provider.stop(&cx)?;

    Ok(())
}
