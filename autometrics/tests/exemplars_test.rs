#![cfg(all(prometheus_exporter, exemplars))]

use autometrics::{autometrics, prometheus_exporter};

#[cfg(exemplars_tracing)]
#[test]
fn single_field() {
    use tracing_subscriber::prelude::*;
    prometheus_exporter::try_init().ok();

    #[autometrics]
    #[tracing::instrument(fields(trace_id = "test_trace_id"))]
    fn single_field_fn() {}

    let subscriber = tracing_subscriber::fmt::fmt().finish().with(
        autometrics::exemplars::tracing::AutometricsExemplarExtractor::from_fields(&["trace_id"]),
    );
    tracing::subscriber::with_default(subscriber, || single_field_fn());

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_total{")
            && line.contains(r#"function="single_field_fn""#)
            && line.ends_with(r#"} 1 # {trace_id="test_trace_id"} 1.0"#)
    }))
}

#[cfg(exemplars_tracing)]
#[test]
fn multiple_fields() {
    use tracing_subscriber::prelude::*;
    prometheus_exporter::try_init().ok();

    #[autometrics]
    #[tracing::instrument(fields(trace_id = "test_trace_id", foo = 99))]
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
        line.starts_with("function_calls_total{")
            && line.contains(r#"function="multiple_fields_fn""#)
            && (line.ends_with(r#"} 1 # {trace_id="test_trace_id",foo="99"} 1.0"#)
                || line.ends_with(r#"} 1 # {foo="99",trace_id="test_trace_id"} 1.0"#))
    }))
}

#[cfg(exemplars_tracing_opentelemetry)]
#[test]
fn tracing_opentelemetry_context() {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_stdout::SpanExporter;
    use std::io;
    use tracing_subscriber::{layer::SubscriberExt, Registry};

    prometheus_exporter::try_init().ok();

    let exporter = SpanExporter::builder().with_writer(io::sink()).build();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("test");

    // This adds the OpenTelemetry Context to every tracing Span
    #[cfg(feature = "exemplars-tracing-opentelemetry-0_31")]
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(otel_layer);

    #[autometrics]
    #[tracing::instrument]
    fn opentelemetry_context_fn() {}

    tracing::subscriber::with_default(subscriber, opentelemetry_context_fn);

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_total{")
            && line.contains(r#"function="opentelemetry_context_fn""#)
            && (line.contains(r#"trace_id=""#) || line.contains(r#"span_id=""#))
    }))
}
