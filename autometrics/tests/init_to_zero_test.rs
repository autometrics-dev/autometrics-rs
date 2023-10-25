#![cfg(feature = "prometheus-exporter")]

use autometrics::{autometrics, prometheus_exporter};

#[cfg(debug_assertions)]
#[test]
fn zero_metrics() {
    // This test is in its own file because there is a race condition when multiple tests
    // are concurrently calling prometheus_exporter::try_init. One of the tests will
    // initialize the exporter and set the global OnceCell while the others are blocked.
    // The thread that initialized the exporter will then set the metrics to zero. However,
    // this test may already try to read the metrics before they are set to zero.
    prometheus_exporter::init();

    #[autometrics]
    fn zero_metrics_fn() {}

    // Note that we are not calling the function, but it should still be registered

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    println!("{}", metrics);
    assert!(metrics
        .lines()
        .any(|line| line.contains(r#"function="zero_metrics_fn""#) && line.ends_with("} 0")));
}
