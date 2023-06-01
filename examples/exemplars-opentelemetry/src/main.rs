use autometrics::{autometrics, prometheus_exporter};
use autometrics_example_util::run_prometheus;
use axum::{routing::get, Router};
use opentelemetry::sdk::export::trace::stdout;
use std::{io, net::SocketAddr, time::Duration};
use tracing::{error, span};
use tracing::{instrument, trace};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

// Autometrics looks for a field called `trace_id` and attaches
// that as an exemplar for the metrics it generates.
#[autometrics]
async fn outer_function() -> String {
    trace!("Outer function called");
    inner_function("hello");

    dbg!(axum_tracing_opentelemetry::find_current_trace_id());
    "Hello world!".to_string()
}

// This function will also have the `trace_id` attached as an exemplar
// because it is called within the same span as `outer_function`.
#[autometrics]
#[instrument]
fn inner_function(param: &str) {
    trace!("Inner function called");
}

// pub fn build_otel_layer<S>() -> Result<OpenTelemetryLayer<S, Tracer>, BoxError>
// where
// S: Subscriber + for<'a> LookupSpan<'a>,
// {
// use crate::{
// init_propagator, //stdio,
// otlp,
// resource::DetectResource,
// };
// let otel_rsrc = DetectResource::default()
// //.with_fallback_service_name(env!("CARGO_PKG_NAME"))
// //.with_fallback_service_version(env!("CARGO_PKG_VERSION"))
// .build();
// let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity)?;
// // to not send trace somewhere, but continue to create and propagate,...
// // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
// // or to `std::io::stdout()` to print
// //
// // let otel_tracer =
// //     stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())?;
// init_propagator()?;
// Ok(tracing_opentelemetry::layer().with_tracer(otel_tracer))
// }

#[tokio::main]
async fn main() {
    // Run Prometheus with flag --enable-feature=exemplars-storage
    let _prometheus = run_prometheus(true);
    tokio::spawn(generate_random_traffic());

    prometheus_exporter::init();

    let tracer = stdout::new_pipeline()
        .with_writer(io::sink())
        .install_simple();

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::fmt().finish().with(telemetry).init();

    let app = Router::new()
        .route("/", get(outer_function))
        .route(
            "/metrics",
            // Expose the metrics to Prometheus in the OpenMetrics format
            get(|| async { prometheus_exporter::encode_http_response() }),
        )
        .layer(axum_tracing_opentelemetry::opentelemetry_tracing_layer());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum::Server::bind(&addr);

    println!("\nVisit the following URL to see one of the charts along with the exemplars:");
    println!("http://localhost:9090/graph?g0.expr=%23%20Rate%20of%20calls%20to%20the%20%60outer_function%60%20function%20per%20second%2C%20averaged%20over%205%20minute%20windows%0A%0Asum%20by%20(function%2C%20module%2C%20commit%2C%20version)%20(rate(%7B__name__%3D~%22function_calls(_count)%3F(_total)%3F%22%2Cfunction%3D%22outer_function%22%7D%5B5m%5D)%20*%20on%20(instance%2C%20job)%20group_left(version%2C%20commit)%20last_over_time(build_info%5B1s%5D))&g0.tab=0&g0.stacked=0&g0.show_exemplars=1&g0.range_input=1h");

    server
        .serve(app.into_make_service())
        .await
        .expect("Error starting example API server");
}

pub async fn generate_random_traffic() {
    let client = reqwest::Client::new();
    loop {
        client.get("http://localhost:3000").send().await.ok();
        tokio::time::sleep(Duration::from_millis(100)).await
    }
}
