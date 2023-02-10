use crate::database::Database;
use crate::util::generate_random_traffic;
use autometrics::{autometrics, generate_alerts, global_metrics_exporter};
use autometrics_example_util::run_prometheus;
use axum::http::Request;
use axum::middleware::{self, Next};
use axum::routing::{get, post};
use axum::{response::Response, Router};
use clap::Parser;
use std::net::SocketAddr;

mod database;
mod error;
mod routes;
mod util;

#[derive(Parser)]
enum Cli {
    /// Run the API
    Serve(ServeArgs),

    /// Generate Prometheus recording and alerting rules
    GenerateAlerts(GenerateAlertsArgs),
}

#[derive(Parser)]
struct ServeArgs {
    /// The port to listen on
    #[clap(short, long, default_value = "3000")]
    port: u16,
}

#[derive(Parser)]
struct GenerateAlertsArgs {
    /// Optional name of file to output the YAML to
    /// (default is to print to stdout)
    #[clap(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args {
        Cli::Serve(args) => handle_serve(args).await,
        Cli::GenerateAlerts(args) => {
            let output = generate_alerts();
            if let Some(filename) = args.output {
                std::fs::write(filename, output).unwrap();
            } else {
                println!("{}\n", output);
            }
        }
    }
}

/// This middleware is applied to all requests so that we can track an overall success rate for the alerts
#[autometrics(ok_if = is_success, alerts(success_rate = 99.9%, latency(99% <= 250ms)))]
async fn all_requests_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    next.run(request).await
}

/// This function is used to determine whether a request was successful or not
fn is_success(response: &Response) -> bool {
    response.status().is_success()
}

/// Run the API server as well as Prometheus and a traffic generator
async fn handle_serve(args: ServeArgs) {
    // Run Prometheus and generate random traffic for the app
    // (You would not actually do this in production, but it makes it easier to see the example in action)
    run_prometheus();
    tokio::spawn(generate_random_traffic());

    // Set up the exporter to collect metrics
    let _exporter = global_metrics_exporter();

    let app = Router::new()
        .route("/", get(routes::get_index))
        .route("/users", post(routes::create_user))
        .route("/random-error", get(routes::get_random_error))
        // Expose the metrics for Prometheus to scrape
        .route("/metrics", get(routes::get_metrics))
        .layer(middleware::from_fn(all_requests_middleware))
        .with_state(Database::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
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
