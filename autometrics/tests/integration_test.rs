#![cfg(prometheus_exporter)]
use autometrics::{autometrics, prometheus_exporter};

#[test]
fn single_function() {
    prometheus_exporter::init();

    #[autometrics]
    fn hello_world() -> &'static str {
        "Hello world!"
    }

    hello_world();
    hello_world();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="hello_world""#)
            && line.contains(r#"module="integration_test""#)
            && line.ends_with("} 2")
    }));
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("function_calls_duration_bucket{")
            && line.contains(r#"function="hello_world""#)
            && line.contains(r#"module="integration_test""#)
            && line.ends_with("} 2")));
}

#[test]
fn impl_block() {
    prometheus_exporter::init();

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

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="test_fn""#)
            && line.ends_with("} 1")
    }));
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("function_calls_duration_bucket{")
            && line.contains(r#"function="test_fn""#)
            && line.ends_with("} 1")));

    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="test_method""#)
            && line.ends_with("} 1")
    }));
    assert!(metrics
        .lines()
        .any(|line| line.starts_with("function_calls_duration_bucket{")
            && line.contains(r#"function="test_method""#)
            && line.ends_with("} 1")));
}

#[test]
fn result() {
    prometheus_exporter::init();

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

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics
        .lines()
        .any(|line| (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="result_fn""#)
            && line.contains(r#"result="error""#)
            && line.ends_with("} 2")));
    assert!(metrics
        .lines()
        .any(|line| (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="result_fn""#)
            && line.contains(r#"result="ok""#)
            && line.ends_with("} 1")));
}

#[test]
fn ok_if() {
    prometheus_exporter::init();

    #[autometrics(ok_if = Option::is_some)]
    fn ok_if_fn() -> Option<()> {
        None
    }

    ok_if_fn();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="ok_if_fn""#)
            && line.contains(r#"result="error""#)
            && line.ends_with("} 1")
    }));
}

#[test]
fn error_if() {
    prometheus_exporter::init();

    #[autometrics(error_if = Option::is_none)]
    fn error_if_fn() -> Option<()> {
        None
    }

    error_if_fn();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"function="error_if_fn""#)
            && line.contains(r#"result="error""#)
            && line.ends_with("} 1")
    }));
}

#[test]
fn caller_label() {
    prometheus_exporter::init();

    #[autometrics]
    fn function_1() {
        function_2()
    }

    #[autometrics]
    fn function_2() {}

    function_1();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| {
        (line.starts_with("function_calls_count{")
            || line.starts_with("function_calls_count_total{"))
            && line.contains(r#"caller="function_1""#)
            && line.contains(r#"function="function_2""#)
            && line.ends_with("} 1")
    }));
}

#[test]
fn build_info() {
    prometheus_exporter::init();

    #[autometrics]
    fn function_just_to_initialize_build_info() {}

    function_just_to_initialize_build_info();
    function_just_to_initialize_build_info();

    let metrics = prometheus_exporter::encode_to_string().unwrap();
    assert!(metrics.lines().any(|line| line.starts_with("build_info{")
        && line.contains(r#"branch="""#)
        && line.contains(r#"commit="""#)
        && line.contains(&format!("version=\"{}\"", env!("CARGO_PKG_VERSION")))
        && line.ends_with("} 1")));
}
