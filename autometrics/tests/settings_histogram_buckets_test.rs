#![cfg(prometheus_exporter)]

use autometrics::{autometrics, prometheus_exporter, settings::AutometricsSettingsBuilder};

#[test]
fn custom_histogram_buckets() {
    #[autometrics]
    fn custom_histogram_buckets_fn() -> &'static str {
        "Hello world!"
    }

    AutometricsSettingsBuilder::default()
        .histogram_buckets(vec![0.1, 0.2, 0.3, 0.4, 0.5])
        .init();

    custom_histogram_buckets_fn();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| line.contains(r#"le="0.1"#)));
    assert!(metrics.lines().any(|line| line.contains(r#"le="0.2"#)));
    assert!(metrics.lines().any(|line| line.contains(r#"le="0.3"#)));
    assert!(metrics.lines().any(|line| line.contains(r#"le="0.4"#)));
    assert!(metrics.lines().any(|line| line.contains(r#"le="0.5"#)));
}
