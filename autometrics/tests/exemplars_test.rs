#![cfg(all(feature = "exemplars-tracing", feature = "prometheus-exporter"))]

use autometrics::exemplars::tracing::AutometricsExemplarExtractor;
use autometrics::{autometrics, prometheus_exporter};
use tracing::instrument;
use tracing_subscriber::prelude::*;

#[test]
fn single_field() {
    prometheus_exporter::init();

    #[autometrics]
    #[instrument(fields(trace_id = "test_trace_id"))]
    fn single_field_fn() {}

    let subscriber = tracing_subscriber::fmt::fmt()
        .finish()
        .with(AutometricsExemplarExtractor::from_fields(&["trace_id"]));
    tracing::subscriber::with_default(subscriber, || single_field_fn());

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_count_total{")
            && line.contains(r#"function="single_field_fn""#)
            && line.ends_with(r#"} 1 # {trace_id="test_trace_id"} 1.0"#)
    }))
}

#[test]
fn multiple_fields() {
    prometheus_exporter::init();

    #[autometrics]
    #[instrument(fields(trace_id = "test_trace_id", foo = 99))]
    fn multiple_fields_fn() {}

    let subscriber =
        tracing_subscriber::fmt::fmt()
            .finish()
            .with(AutometricsExemplarExtractor::from_fields(&[
                "trace_id", "foo",
            ]));
    tracing::subscriber::with_default(subscriber, || multiple_fields_fn());

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    println!("{}", metrics);
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_count_total{")
            && line.contains(r#"function="multiple_fields_fn""#)
            && (line.ends_with(r#"} 1 # {trace_id="test_trace_id",foo="99"} 1.0"#)
                || line.ends_with(r#"} 1 # {foo="99",trace_id="test_trace_id"} 1.0"#))
    }))
}
