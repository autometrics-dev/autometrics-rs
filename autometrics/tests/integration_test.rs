use autometrics::autometrics;
use regex::Regex;

#[cfg(feature = "prometheus-exporter")]
#[test]
fn single_function() {
    let _ = autometrics::global_metrics_exporter();

    #[autometrics]
    fn hello_world() -> &'static str {
        "Hello world!"
    }

    hello_world();
    hello_world();

    let metrics = autometrics::encode_global_metrics().unwrap();
    println!("{}", metrics);
    let call_count_metric: Regex = Regex::new(
        r#"function_calls_count\{\S*function="hello_world"\S*module="integration_test"\S*\} 2"#,
    )
    .unwrap();
    let duration_metric: Regex = Regex::new(
        r#"function_calls_duration_bucket\{\S*function="hello_world"\S*module="integration_test"\S*\}"#,
    )
    .unwrap();
    assert!(call_count_metric.is_match(&metrics));
    assert!(duration_metric.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn impl_block() {
    let _ = autometrics::global_metrics_exporter();

    struct Foo;

    #[autometrics]
    impl Foo {
        fn test_fn() -> &'static str {
            "Hello world!"
        }

        fn test_method(&self) -> &'static str {
            "Goodnight moon"
        }
    }

    Foo::test_fn();
    Foo.test_method();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let test_fn_count: Regex =
        Regex::new(r#"function_calls_count\{\S*function="test_fn"\S*\} 1"#).unwrap();
    let test_method_count: Regex =
        Regex::new(r#"function_calls_count\{\S*function="test_method"\S*\} 1"#).unwrap();
    let test_fn_duration: Regex =
        Regex::new(r#"function_calls_duration_bucket\{\S*function="test_fn"\S*\}"#).unwrap();
    let test_method_duration: Regex =
        Regex::new(r#"function_calls_duration_bucket\{\S*function="test_method"\S*\}"#).unwrap();
    assert!(test_fn_count.is_match(&metrics));
    assert!(test_method_count.is_match(&metrics));
    assert!(test_fn_duration.is_match(&metrics));
    assert!(test_method_duration.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn result() {
    let _ = autometrics::global_metrics_exporter();

    #[autometrics]
    fn result_fn(should_error: bool) -> Result<(), ()> {
        if should_error {
            Err(())
        } else {
            Ok(())
        }
    }

    result_fn(true).ok();
    result_fn(true).ok();
    result_fn(false).ok();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let error_count: Regex =
        Regex::new(r#"function_calls_count\{\S*function="result_fn"\S*result="error"\S*\} 2"#)
            .unwrap();
    assert!(error_count.is_match(&metrics));
    let ok_count: Regex =
        Regex::new(r#"function_calls_count\{\S*function="result_fn"\S*result="ok"\S*\} 1"#)
            .unwrap();
    assert!(ok_count.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn ok_if() {
    let _ = autometrics::global_metrics_exporter();

    #[autometrics(ok_if = Option::is_some)]
    fn ok_if_fn() -> Option<()> {
        None
    }

    ok_if_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let error_count: Regex =
        Regex::new(r#"function_calls_count\{\S*function="ok_if_fn"\S*result="error"\S*\} 1"#)
            .unwrap();
    assert!(error_count.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn error_if() {
    let _ = autometrics::global_metrics_exporter();

    #[autometrics(error_if = Option::is_none)]
    fn error_if_fn() -> Option<()> {
        None
    }

    error_if_fn();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let error_count: Regex =
        Regex::new(r#"function_calls_count\{\S*function="error_if_fn"\S*result="error"\S*\} 1"#)
            .unwrap();
    assert!(error_count.is_match(&metrics));
}

#[cfg(feature = "prometheus-exporter")]
#[test]
fn caller_label() {
    let _ = autometrics::global_metrics_exporter();

    #[autometrics]
    fn function_1() {
        function_2()
    }

    #[autometrics]
    fn function_2() {}

    function_1();

    let metrics = autometrics::encode_global_metrics().unwrap();
    let call_count: Regex = Regex::new(
        r#"function_calls_count\{\S*caller="function_1"\S*function="function_2"\S*\} 1"#,
    )
    .unwrap();
    assert!(call_count.is_match(&metrics));
}
