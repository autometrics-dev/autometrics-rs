#![cfg(prometheus_exporter)]

use autometrics::{autometrics, prometheus_exporter, settings::AutometricsSettings};

#[test]
fn set_service_name() {
    #[autometrics]
    fn test_fn() -> &'static str {
        "Hello world!"
    }

    AutometricsSettings::new()
        .service_name("test_service")
        .init();
    prometheus_exporter::init();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics
        .lines()
        .any(|line| line.contains(r#"service_name="test_service""#)));
}
