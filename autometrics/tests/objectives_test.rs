use autometrics::{autometrics, objectives::*};
use regex::Regex;

#[cfg(feature = "prometheus-exporter")]
#[test]
fn success_rate() {
    let _ = autometrics::global_metrics_exporter();

    const OBJECTIVE: Objective = Objective::new("test").success_rate(ObjectivePercentile::P99);

    #[autometrics(objective = OBJECTIVE)]
    fn test_fn() -> &'static str {
        "Hello world!"
    }

    test_fn();
    test_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let call_count_metric: Regex = Regex::new(
        r#"function_calls_count\{\S*function="test_fn"\S*objective_name="test",objective_percentile="99"\S*\} 2"#,
    )
    .unwrap();
    assert!(call_count_metric.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn latency() {
    let _ = autometrics::global_metrics_exporter();

    const OBJECTIVE: Objective =
        Objective::new("test").latency(ObjectiveLatency::Ms100, ObjectivePercentile::P99_9);

    #[autometrics(objective = OBJECTIVE)]
    fn test_fn() -> &'static str {
        "Hello world!"
    }

    test_fn();
    test_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let duration_metric: Regex = Regex::new(
        r#"function_calls_duration_bucket\{\S*function="test_fn"\S*objective_latency_threshold="0.1",objective_name="test",objective_percentile="99.9"\S*\} 2"#,
    )
    .unwrap();
    assert!(duration_metric.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn combined_objective() {
    let _ = autometrics::global_metrics_exporter();

    const OBJECTIVE: Objective = Objective::new("test")
        .success_rate(ObjectivePercentile::P99)
        .latency(ObjectiveLatency::Ms100, ObjectivePercentile::P99_9);

    #[autometrics(objective = OBJECTIVE)]
    fn test_fn() -> &'static str {
        "Hello world!"
    }

    test_fn();
    test_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let call_count_metric: Regex = Regex::new(
        r#"function_calls_count\{\S*function="test_fn"\S*objective_name="test",objective_percentile="99"\S*\} 2"#,
    )
    .unwrap();
    let duration_metric: Regex = Regex::new(
        r#"function_calls_duration_bucket\{\S*function="test_fn"\S*objective_latency_threshold="0.1",objective_name="test",objective_percentile="99.9"\S*\} 2"#,
    )
    .unwrap();
    assert!(call_count_metric.is_match(&metrics));
    assert!(duration_metric.is_match(&metrics));
}
