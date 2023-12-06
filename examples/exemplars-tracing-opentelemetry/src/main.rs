use autometrics::{autometrics, prometheus_exporter};
use autometrics_example_util::run_prometheus;
use axum::{routing::get, Router, Server, ServiceExt};
use opentelemetry::sdk::trace::TracerProvider;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_stdout::SpanExporter;
use std::error::Error;
use std::net::Ipv4Addr;
use std::{io, net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tracing::{instrument, trace};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, prelude::*, Registry};

// The instrument macro creates a new span for every call to this function,
// and the OpenTelemetryLayer added below attaches the OpenTelemetry Context
// to every span.
//
// Autometrics will pick up that Context and create exemplars from it.
#[autometrics]
#[instrument]
async fn outer_function() {
    inner_function();
}

// This function will also have exemplars because it is called within
// the span of the outer_function
#[autometrics]
fn inner_function() {
    trace!("Inner function called");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Run Prometheus with flag --enable-feature=exemplars-storage
    let _prometheus = run_prometheus(true);
    tokio::spawn(generate_random_traffic());

    // This exporter will discard the spans but you can use the other to see them
    let exporter = SpanExporter::builder().with_writer(io::sink()).build();
    // let exporter = SpanExporter::default();

    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("example");

    // This adds the OpenTelemetry Context to every tracing Span
    let otel_layer = OpenTelemetryLayer::new(tracer);
    Registry::default().with(otel_layer).init();

    prometheus_exporter::init();

    let app = Router::new().route("/", get(outer_function)).route(
        "/metrics",
        // Expose the metrics to Prometheus in the OpenMetrics format
        get(|| async { prometheus_exporter::encode_http_response() }),
    );

    println!("\nVisit the following URL to see one of the charts along with the exemplars:");
    println!("http://localhost:9090/graph?g0.expr=%23%20Rate%20of%20calls%20to%20the%20%60outer_function%60%20function%20per%20second%2C%20averaged%20over%205%20minute%20windows%0A%0Asum%20by%20(function%2C%20module%2C%20commit%2C%20version)%20(rate(%7B__name__%3D~%22function_calls(_count)%3F(_total)%3F%22%2Cfunction%3D%22outer_function%22%7D%5B5m%5D)%20*%20on%20(instance%2C%20job)%20group_left(version%2C%20commit)%20last_over_time(build_info%5B1s%5D))&g0.tab=0&g0.stacked=0&g0.show_exemplars=1&g0.range_input=1h");

    let listener = TcpListener::bind((Ipv4Addr::from([127, 0, 0, 1]), 3000)).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

pub async fn generate_random_traffic() {
    let client = reqwest::Client::new();
    loop {
        client.get("http://localhost:3000").send().await.ok();
        tokio::time::sleep(Duration::from_millis(100)).await
    }
}
