use actix_web::http::StatusCode;
use actix_web::{get, App, HttpServer, Responder};
use autometrics::autometrics;
use autometrics_example_util::{run_prometheus, sleep_random_duration};
use rand::{random, thread_rng, Rng};
use std::io;
use std::time::Duration;
use tokio::time::sleep;

#[get("/")]
#[autometrics]
async fn index_get() -> &'static str {
    "Hello world!"
}

#[get("/random-error")]
#[autometrics]
async fn random_error_get() -> Result<&'static str, io::Error> {
    let should_error: bool = random();

    sleep_random_duration().await;

    if should_error {
        Err(io::Error::new(io::ErrorKind::Other, "its joever"))
    } else {
        Ok("ok")
    }
}

/// This function doesn't return a Result, but we can determine whether
/// we want to consider it a success or not by passing a function to the `ok_if` parameter.
#[autometrics(ok_if = is_success)]
pub async fn route_that_returns_responder() -> impl Responder {
    ("Hello world!", StatusCode::OK)
}

/// Determine whether the response was a success or not
fn is_success<R: Responder>(_: &R) -> bool {
    random()
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let address = "127.0.0.1:3000";

    // Run Prometheus and generate random traffic for the app
    // (You would not actually do this in production, but it makes it easier to see the example in action)
    let _prometheus = run_prometheus(false);
    tokio::spawn(generate_random_traffic());

    println!(
        "The example API server is now running on: {address} \n\
         Wait a few seconds for the traffic generator to create some fake traffic. \n\
         Then, hover over one of the HTTP handler functions (in your editor) to bring up the Rust Docs. \n\
         Click on one of the Autometrics links to see the graph for that handler's metrics in Prometheus."
    );

    HttpServer::new(|| App::new().service(index_get).service(random_error_get))
        .bind(address)?
        .run()
        .await
}

/// Make some random API calls to generate data that we can see in the graphs
async fn generate_random_traffic() {
    loop {
        let sleep_duration = Duration::from_millis(thread_rng().gen_range(10..50));

        let url = match thread_rng().gen_range(0..2) {
            0 => "http://localhost:3000",
            1 => "http://localhost:3000/random-error",
            _ => unreachable!(),
        };

        let _ = reqwest::get(url).await;
        sleep(sleep_duration).await
    }
}
