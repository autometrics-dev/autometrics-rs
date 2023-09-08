use autometrics::{autometrics, otel_push_exporter};
use autometrics_example_util::sleep_random_duration;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[autometrics]
async fn do_stuff() {
    println!("Doing stuff...");
    sleep_random_duration().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // NOTICE: the variable gets assigned to `_meter_provider` instead of just `_`, as the later case
    // would cause it to be dropped immediately and thus shut down.
    let _meter_provider = otel_push_exporter::init_http("http://0.0.0.0:4318")?;
    // or: otel_push_exporter::init_grpc("http://0.0.0.0:4317");

    for _ in 0..100 {
        do_stuff().await;
    }

    println!("Waiting so that we could see metrics going down...");
    sleep(Duration::from_secs(10)).await;

    // no need to call `.shutdown` as the returned `OtelMeterProvider` has a `Drop` implementation
    Ok(())
}
