use autometrics::{autometrics, objectives::*};

#[cfg(feature = "prometheus-exporter")]
#[test]
fn success_rate() {
    let _ = autometrics::global_metrics_exporter();

    const OBJECTIVE: Objective = Objective::new("test").success_rate(ObjectivePercentile::P99);

    #[autometrics(objective = OBJECTIVE)]
    fn success_rate_fn() -> &'static str {
        "Hello world!"
    }

    success_rate_fn();
    success_rate_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    assert!(metrics
        .lines()
        .any(|line| (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="success_rate_fn""#)
            && line.contains(r#"objective_name="test""#)
            && line.contains(r#"objective_percentile="99""#)
            && line.ends_with("} 2")));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn latency() {
    let _ = autometrics::global_metrics_exporter();

    const OBJECTIVE: Objective =
        Objective::new("test").latency(ObjectiveLatency::Ms100, ObjectivePercentile::P99_9);

    #[autometrics(objective = OBJECTIVE)]
    fn latency_fn() -> &'static str {
        "Hello world!"
    }

    latency_fn();
    latency_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_duration_bucket{")
            && line.contains(r#"function="latency_fn""#)
            && line.contains(r#"objective_latency_threshold="0.1""#)
            && line.contains(r#"objective_name="test""#)
            && line.contains(r#"objective_percentile="99.9""#)
            && line.ends_with("} 2")
    }));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn combined_objective() {
    let _ = autometrics::global_metrics_exporter();

    const OBJECTIVE: Objective = Objective::new("test")
        .success_rate(ObjectivePercentile::P99)
        .latency(ObjectiveLatency::Ms100, ObjectivePercentile::P99_9);

    #[autometrics(objective = OBJECTIVE)]
    fn combined_objective_fn() -> &'static str {
        "Hello world!"
    }

    combined_objective_fn();
    combined_objective_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="combined_objective_fn""#)
            && line.contains(r#"objective_name="test""#)
            && line.contains(r#"objective_percentile="99""#)
            && line.ends_with("} 2")
    }));
    assert!(metrics.lines().any(|line| {
        line.starts_with("function_calls_duration_bucket{")
            && line.contains(r#"function="combined_objective_fn""#)
            && line.contains(r#"objective_latency_threshold="0.1""#)
            && line.contains(r#"objective_name="test""#)
            && line.contains(r#"objective_percentile="99.9""#)
            && line.ends_with("} 2")
    }));
}
