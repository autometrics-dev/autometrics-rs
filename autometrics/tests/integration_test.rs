use autometrics::autometrics;
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry_sdk::export::metrics::aggregation;
use opentelemetry_sdk::metrics::{controllers, processors, selectors};
use prometheus::TextEncoder;

fn init_meter() -> PrometheusExporter {
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([25.0, 50.0, 100.0, 200.0, 500.0, 1000.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    opentelemetry_prometheus::exporter(controller).init()
}

#[test]
fn main() {
    #[derive(PartialEq, Debug)]
    struct Function(&'static str);

    let exporter = init_meter();

    add(1, 2);
    other_function().unwrap();

    let encoder = TextEncoder::new();
    let metric_families = exporter.registry().gather();
    let result = encoder.encode_to_string(&metric_families).unwrap();

    assert_ne!(result, "");
}

#[autometrics]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Example HTTP handler function
#[autometrics(
    alerts(success_rate = 99.9%, latency(99.9% < 200ms)),
)]
pub async fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

#[autometrics(track_concurrency, alerts(latency(99.999% < 50ms)))]
fn other_function() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

pub struct Db {}

#[autometrics]
impl Db {
    pub fn new() -> Self {
        Db {}
    }

    pub fn get_user(&self, id: i32) -> Result<String, ()> {
        Ok(format!("User {}", id))
    }

    pub fn get_users(&self) -> Vec<String> {
        Vec::new()
    }
}

trait Foo<'a> {
    fn foo(&'a self) -> Result<String, ()>;
}

#[autometrics]
impl<'a> Foo<'a> for Db {
    fn foo(&self) -> Result<String, ()> {
        Ok("Bar".to_string())
    }
}
