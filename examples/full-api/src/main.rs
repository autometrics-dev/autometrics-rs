use crate::database::Database;
use crate::util::generate_random_traffic;
use autometrics::global_metrics_exporter;
use autometrics_example_util::run_prometheus;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;

mod database;
mod error;
mod routes;
mod util;

/// Run the API server as well as Prometheus and a traffic generator
#[tokio::main]
async fn main() {
    // Run Prometheus and generate random traffic for the app
    // (You would not actually do this in production, but it makes it easier to see the example in action)
    let _promehtheus = run_prometheus();
    tokio::spawn(generate_random_traffic());

    // Set up the exporter to collect metrics
    let _exporter = global_metrics_exporter();

    let app = Router::new()
        .route("/", get(routes::get_index))
        .route("/users", post(routes::create_user))
        .route("/random-error", get(routes::get_random_error))
        // Expose the metrics for Prometheus to scrape
        .route("/metrics", get(routes::get_metrics))
        .with_state(Database::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum::Server::bind(&addr);

    println!(
        "The example API server is now running on: {addr}

Wait a few seconds for the traffic generator to create some fake traffic.
Then, hover over one of the HTTP handler functions (in your editor) to bring up the Rust Docs.

Click on one of the Autometrics links to see the graph for that handler's metrics in Prometheus."
    );

    server
        .serve(app.into_make_service())
        .await
        .expect("Error starting example API server");
}
