#![cfg(feature = "prometheus-exporter")]

use autometrics::{autometrics, prometheus_exporter};
use tracing::instrument;
use tracing_subscriber::prelude::*;

#[cfg(feature = "exemplars-tracing")]
#[test]
fn single_field() {
    prometheus_exporter::init();

    #[autometrics]
    #[instrument(fields(trace_id = "test_trace_id"))]
    fn single_field_fn() {}

    let subscriber = tracing_subscriber::fmt::fmt().finish().with(
        autometrics::exemplars::tracing::AutometricsExemplarExtractor::from_fields(&["trace_id"]),
    );
    tracing::subscriber::with_default(subscriber, || single_field_fn());

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_count_total{")
            && line.contains(r#"function="single_field_fn""#)
            && line.ends_with(r#"} 1 # {trace_id="test_trace_id"} 1.0"#)
    }))
}

#[cfg(feature = "exemplars-tracing")]
#[test]
fn multiple_fields() {
    prometheus_exporter::init();

    #[autometrics]
    #[instrument(fields(trace_id = "test_trace_id", foo = 99))]
    fn multiple_fields_fn() {}

    let subscriber = tracing_subscriber::fmt::fmt().finish().with(
        autometrics::exemplars::tracing::AutometricsExemplarExtractor::from_fields(&[
            "trace_id", "foo",
        ]),
    );
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

#[cfg(feature = "exemplars-opentelemetry")]
#[test]
fn opentelemetry_context() {
    use opentelemetry_api::trace::Tracer;
    prometheus_exporter::init();

    #[autometrics]
    fn opentelemetry_context_fn() {}

    let tracer = opentelemetry_sdk::export::trace::stdout::new_pipeline().install_simple();
    tracer.in_span("my_span", |_cx| opentelemetry_context_fn());

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_count_total{")
            && line.contains(r#"function="opentelemetry_context_fn""#)
            && (line.contains(r#"trace_id=""#) || line.contains(r#"span_id=""#))
    }))
}
