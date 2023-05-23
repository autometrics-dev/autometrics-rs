![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

Autometrics is an observability micro-framework built for developers.

The Rust library provides a macro that makes it easy to instrument any function with the most useful metrics: request rate, error rate, and latency. Autometrics uses instrumented function names to generate Prometheus queries so you don’t need to hand-write complicated PromQL.

To make it easy for you to spot and debug issues in production, Autometrics inserts links to live charts directly into each function’s doc comments and provides dashboards that work out of the box. It also enables you to create powerful alerts based on Service-Level Objectives (SLOs) directly in your source code. Lastly, Autometrics writes queries that correlate your software’s version info with anomalies in the metrics to help you quickly identify commits that introduced bugs or latency.

## Example Axum App

Autometrics isn't tied to any web framework, but this shows how you can use the library in an [Axum](https://github.com/tokio-rs/axum) server.

```rust
use autometrics::autometrics;
use axum::{http::StatusCode, routing::*, Router, Server};

// 1. Instrument your functions with metrics
#[autometrics]
pub async fn create_user() -> Result<(), ()> {
  Ok(())
}

// 2. Export your metrics to Prometheus
pub async fn get_metrics() -> (StatusCode, String) {
  match autometrics::encode_global_metrics() {
    Ok(metrics) => (StatusCode::OK, metrics),
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
  }
}

// 3. Initialize the metrics collector in your main function
#[tokio::main]
pub async fn main() {
  let _exporter = autometrics::global_metrics_exporter();

  let app = Router::new()
      .route("/users", post(create_user))
      .route("/metrics", get(get_metrics));
  Server::bind(&([127, 0, 0, 1], 0).into())
      .serve(app.into_make_service());
}
```

## Features

- ✨ [`#[autometrics]`](https://docs.rs/autometrics/latest/autometrics/attr.autometrics.html) macro instruments any function or `impl` block to track the [most useful metrics](https://docs.rs/autometrics/latest/autometrics/attr.autometrics.html#generated-metrics)
- 💡 Writes Prometheus queries so you can understand the data generated without knowing PromQL
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- [🔍 Identify commits](#identifying-commits-that-introduced-problems) that introduced errors or increased latency
- [🚨 Define alerts](https://docs.rs/autometrics/latest/autometrics/objectives/index.html) using SLO best practices directly in your source code
- [📊 Grafana dashboards](https://github.com/autometrics-dev#5-configuring-prometheus) work out of the box to visualize the performance of instrumented functions & SLOs
- [⚙️ Configurable](#metrics-libraries) metric collection library ([`opentelemetry`](https://crates.io/crates/opentelemetry), [`prometheus`](https://crates.io/crates/prometheus), or [`metrics`](https://crates.io/crates/metrics))
- ⚡ Minimal runtime overhead

See [Why Autometrics?](https://github.com/autometrics-dev#4-why-autometrics) for more details on the ideas behind autometrics.

## Identifying commits that introduced problems

Autometrics makes it easy to [spot versions and commits that introduce errors or latency](https://fiberplane.com/blog/autometrics-rs-0-4-spot-commits-that-introduce-errors-or-slow-down-your-application).

It produces a `build_info` metric and uses the following labels to expose the version info of your app to Prometheus:

| Label | Compile-Time Environment Variables | Default |
|---|---|---|
| `version` | `AUTOMETRICS_VERSION` or `CARGO_PKG_VERSION` | `CARGO_PKG_VERSION` (set by cargo by default) |
| `commit` | `AUTOMETRICS_COMMIT` or `VERGEN_GIT_COMMIT` | `""` |
| `branch` | `AUTOMETRICS_BRANCH` or `VERGEN_GIT_BRANCH` | `""` |

### (Optional) Using [`vergen`](https://crates.io/crates/vergen) to set the Git details

```toml
# Cargo.toml

[build-dependencies]
vergen = { version = "8.1", features = ["git", "gitcl"] }
```

```rust
// build.rs
fn main() {
  vergen::EmitBuilder::builder()
      .git_sha(true)
      .git_branch()
      .emit()
      .expect("Unable to generate build info");
}
```


## Configuring Autometrics

### Custom Prometheus URL

Autometrics inserts Prometheus query links into function documentation. By default, the links point to `http://localhost:9090` but you can configure it to use a custom URL using an environment variable in your `build.rs` file:

```rust
// build.rs

fn main() {
  // Reload Rust analyzer after changing the Prometheus URL to regenerate the links
  let prometheus_url = "https://your-prometheus-url.example";
  println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
}
```

### Feature flags

- `prometheus-exporter` - exports a Prometheus metrics collector and exporter (compatible with any of the Metrics Libraries)
- `custom-objective-latency` - by default, Autometrics only supports a fixed set of latency thresholds for objectives. Enable this to use custom latency thresholds. Note, however, that the custom latency **must** match one of the buckets configured for your histogram or the alerts will not work. This is not currently compatible with the `prometheus` or `prometheus-exporter` feature.
- `custom-objective-percentile` by default, Autometrics only supports a fixed set of objective percentiles. Enable this to use a custom percentile. Note, however, that using custom percentiles requires generating a different recording and alerting rules file using the CLI + Sloth (see [here](https://github.com/autometrics-dev/autometrics-rs/tree/main/autometrics-cli)).

#### Metrics Libraries

Configure the crate that autometrics will use to produce metrics by using one of the following feature flags:

> **Note**
>
> If you are **not** using the `prometheus-exporter`, you must ensure that you are using the exact same version of the metrics library as `autometrics` (and it must come from `crates.io` rather than git or another source). If not, the autometrics metrics will not appear in your exported metrics.

- `opentelemetry` (enabled by default) - use the [opentelemetry](https://crates.io/crates/opentelemetry) crate for producing metrics.
- `metrics` - use the [metrics](https://crates.io/crates/metrics) crate for producing metrics
- `prometheus` - use the [prometheus](https://crates.io/crates/prometheus) crate for producing metrics
