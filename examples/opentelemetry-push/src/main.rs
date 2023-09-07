use autometrics::autometrics;
use autometrics_example_util::sleep_random_duration;
use opentelemetry::metrics::MetricsError;
use opentelemetry::sdk::metrics::MeterProvider;
use opentelemetry::{runtime, Context};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

fn init_metrics() -> Result<MeterProvider, MetricsError> {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..ExportConfig::default()
    };
    let push_interval = Duration::from_secs(1);
    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
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
    meter_provider.force_flush(&cx)?;

    meter_provider.shutdown()?;
    Ok(())
}
