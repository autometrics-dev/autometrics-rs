use autometrics::{
    autometrics, encode_global_metrics, exemplars::tracing::AutometricsExemplarExtractor,
};
use tracing::{instrument, trace};
use tracing_subscriber::{prelude::*, EnvFilter};
use uuid::Uuid;

// Autometrics looks for a field called `trace_id` and attaches
// that as an exemplar for the metrics it generates.
#[autometrics]
#[instrument(fields(trace_id = %Uuid::new_v4()))]
fn outer_function() {
    trace!("Outer function called");
    inner_function("hello")
}

// This function will also have the `trace_id` attached as an exemplar
// because it is called within the same span as `outer_function`.
#[autometrics]
#[instrument]
fn inner_function(param: &str) {
    trace!("Inner function called");
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .finish()
        .with(EnvFilter::from_default_env())
        .with(AutometricsExemplarExtractor::from_field("trace_id"))
        .init();

    outer_function();

    println!("{}", encode_global_metrics().unwrap());
}