use autometrics::{autometrics, prometheus_exporter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[inline(never)]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[autometrics]
#[inline(never)]
pub fn instrumented_add(a: i32, b: i32) -> i32 {
    a + b
}

#[inline(never)]
pub fn clone_result(input: String) -> Result<String, String> {
    Ok(input.to_string())
}

#[autometrics]
#[inline(never)]
pub fn instrumented_clone_result(input: String) -> Result<String, String> {
    Ok(input.to_string())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    prometheus_exporter::init();

    let backend = if cfg!(metrics) {
        "metrics"
    } else if cfg!(opentelemetry) {
        "opentelemetry"
    } else if cfg!(prometheus) {
        "prometheus"
    } else if cfg!(prometheus_client_feature) {
        "prometheus-client"
    } else {
        "unknown"
    };

    let mut add_group = c.benchmark_group("Add");
    add_group.bench_function("baseline", |b| b.iter(|| add(black_box(20), black_box(30))));
    add_group.bench_function(format!("autometrics + {backend}"), |b| {
        b.iter(|| instrumented_add(black_box(20), black_box(30)))
    });
    add_group.finish();

    let mut clone_result_group = c.benchmark_group("Clone String and return Result");
    clone_result_group.bench_function("baseline", |b| {
        b.iter(|| clone_result(black_box("hello".to_string())))
    });
    clone_result_group.bench_function(format!("autometrics + {backend}"), |b| {
        b.iter(|| instrumented_clone_result(black_box("hello".to_string())));
    });
    clone_result_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
