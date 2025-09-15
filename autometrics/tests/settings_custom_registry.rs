#![cfg(prometheus_exporter)]

use autometrics::{autometrics, prometheus_exporter, settings::AutometricsSettings};

#[cfg(prometheus_client)]
#[test]
fn custom_prometheus_client_registry() {
    use prometheus_client::encoding::text::encode;
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use prometheus_client::registry::Registry;

    #[autometrics]
    fn hello_world() -> &'static str {
        "Hello world!"
    }

    // Create our own registry
    let mut registry = <Registry>::default();

    // Also create a custom metric
    let custom_metric = Family::<Vec<(&str, &str)>, Counter>::default();
    registry.register("custom_metric", "My custom metric", custom_metric.clone());

    let settings = AutometricsSettings::builder()
        .prometheus_client_registry(registry)
        .init();

    // Increment the custom metric
    custom_metric.get_or_create(&vec![("foo", "bar")]).inc();

    hello_world();

    let mut metrics = String::new();
    encode(&mut metrics, settings.prometheus_client_registry()).unwrap();

    // Check that both the autometrics metrics and the custom metrics are present
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("function_calls_total{")
            && line.contains(r#"function="hello_world""#)));
    assert!(metrics
        .lines()
        .any(|line| line == "custom_metric_total{foo=\"bar\"} 1"));

    // The output of the prometheus_exporter should be the same
    assert_eq!(metrics, prometheus_exporter::encode_to_string().unwrap());
}

#[cfg(prometheus)]
#[test]
fn custom_prometheus_registry() {
    use prometheus::{register_counter_vec_with_registry, Registry, TextEncoder};
    let registry = Registry::new();

    let custom_metric = register_counter_vec_with_registry!(
        "custom_metric",
        "My custom metric",
        &["foo"],
        registry.clone()
    )
    .unwrap();

    let settings = AutometricsSettings::builder()
        .prometheus_registry(registry)
        .init();

    #[autometrics]
    fn hello_world() -> &'static str {
        "Hello world!"
    }

    hello_world();
    custom_metric.with_label_values(&["bar"]).inc();

    let mut metrics = String::new();
    TextEncoder::new()
        .encode_utf8(&settings.prometheus_registry().gather(), &mut metrics)
        .unwrap();

    // Check that both the autometrics metrics and the custom metrics are present
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("function_calls_total{")
            && line.contains(r#"function="hello_world""#)));
    assert!(metrics
        .lines()
        .any(|line| line == "custom_metric{foo=\"bar\"} 1"));

    // The output of the prometheus_exporter should be the same
    assert_eq!(metrics, prometheus_exporter::encode_to_string().unwrap());
}

#[cfg(opentelemetry)]
#[test]
fn custom_opentelemetry_registry() {
    use opentelemetry::{global, KeyValue};
    use prometheus::{Registry, TextEncoder};

    // OpenTelemetry uses the `prometheus` crate under the hood
    let registry = Registry::new();

    let settings = AutometricsSettings::builder()
        .prometheus_registry(registry)
        .init();

    let custom_metric = global::meter("foo").u64_counter("custom_metric").build();

    #[autometrics]
    fn hello_world() -> &'static str {
        "Hello world!"
    }

    hello_world();
    custom_metric.add(1, &[KeyValue::new("foo", "bar")]);

    let mut metrics = String::new();
    TextEncoder::new()
        .encode_utf8(&settings.prometheus_registry().gather(), &mut metrics)
        .unwrap();

    // Check that both the autometrics metrics and the custom metrics are present
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("function_calls_total{")
            && line.contains(r#"function="hello_world""#)));
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("custom_metric_total{")
            && line.contains("foo=\"bar\"")
            && line.ends_with("} 1")));

    // The output of the prometheus_exporter should be the same
    assert_eq!(metrics, prometheus_exporter::encode_to_string().unwrap());
}
