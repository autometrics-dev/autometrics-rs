use autometrics::{autometrics, encode_global_metrics, integrations::tracing::AutometricsLayer};
use tracing::{instrument, trace};
use tracing_subscriber::{prelude::*, EnvFilter};
use uuid::Uuid;

// Autometrics looks for a field called `trace_id` and attaches
// that as an exemplar for the metrics it generates.
#[autometrics]
#[instrument(fields(trace_id = %Uuid::new_v4(), foo = "bar"))]
fn outer_function() {
    trace!("Outer function called");
    inner_function("hello")
}

#[autometrics]
#[instrument]
fn inner_function(param: &str) {
    trace!("Inner function called");
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .finish()
        .with(EnvFilter::from_default_env())
        .with(AutometricsLayer::default())
        .init();

    for _i in 0..10 {
        outer_function();
    }

    println!("{}", encode_global_metrics().unwrap());
}
