use autometrics::{
    autometrics, exemplars::tracing::AutometricsExemplarExtractor, prometheus_exporter,
};
use autometrics_example_util::run_prometheus;
use axum::{routing::get, Router};
use std::error::Error;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{instrument, trace};
use tracing_subscriber::{prelude::*, EnvFilter};
use uuid::Uuid;

// Autometrics looks for a field called `trace_id` and attaches
// that as an exemplar for the metrics it generates.
#[autometrics]
#[instrument(fields(trace_id = %Uuid::new_v4()))]
async fn outer_function() -> String {
    trace!("Outer function called");
    inner_function("hello");
    "Hello world!".to_string()
}

// This function will also have the `trace_id` attached as an exemplar
// because it is called within the same span as `outer_function`.
#[autometrics]
#[instrument]
fn inner_function(param: &str) {
    trace!("Inner function called");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Run Prometheus with flag --enable-feature=exemplars-storage
    let _prometheus = run_prometheus(true);
    tokio::spawn(generate_random_traffic());

    prometheus_exporter::init();
    tracing_subscriber::fmt::fmt()
        .finish()
        .with(EnvFilter::from_default_env())
        // Set up the tracing layer to expose the `trace_id` field to Autometrics.
        .with(AutometricsExemplarExtractor::from_fields(&["trace_id"]))
        .init();

    let app = Router::new().route("/", get(outer_function)).route(
        "/metrics",
        // Expose the metrics to Prometheus in the OpenMetrics format
        get(|| async { prometheus_exporter::encode_http_response() }),
    );

    println!("\nVisit the following URL to see one of the charts along with the exemplars:");
    println!("http://localhost:9090/graph?g0.expr=%23%20Rate%20of%20calls%20to%20the%20%60outer_function%60%20function%20per%20second%2C%20averaged%20over%205%20minute%20windows%0A%0Asum%20by%20(function%2C%20module%2C%20commit%2C%20version)%20(rate(%7B__name__%3D~%22function_calls(_count)%3F(_total)%3F%22%2Cfunction%3D%22outer_function%22%7D%5B5m%5D)%20*%20on%20(instance%2C%20job)%20group_left(version%2C%20commit)%20last_over_time(build_info%5B1s%5D))&g0.tab=0&g0.stacked=0&g0.show_exemplars=1&g0.range_input=1h");

    let listener = TcpListener::bind((Ipv4Addr::from([127, 0, 0, 1]), 3000)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn generate_random_traffic() {
    let client = reqwest::Client::new();
    loop {
        client.get("http://localhost:3000").send().await.ok();
        tokio::time::sleep(Duration::from_millis(100)).await
    }
}
