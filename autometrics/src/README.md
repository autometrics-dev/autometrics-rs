<!-- This is used on docs.rs -->

![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

Autometrics provides a macro that makes it easy to instrument any function with the most useful metrics: request rate, error rate, and latency. It then uses the instrumented function names to generate Prometheus queries to help you identify and debug issues in production.

## Features

- ✨ [`#[autometrics]`](autometrics) macro instruments any function or `impl` block to track the [most useful metrics](#generated-metrics)
- 💡 Writes Prometheus queries so you can understand the data generated without knowing PromQL
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- [🔍 Identify commits](#build_info-metric-labels) that introduced errors or increased latency
- [🚨 Define alerts](objectives) using SLO best practices directly in your source code
- [📊 Grafana dashboards](https://github.com/autometrics-dev#5-configuring-prometheus) work out of the box to visualize the performance of instrumented functions & SLOs
- [⚙️ Configurable](#metrics-libraries) metric collection library ([`opentelemetry`](https://crates.io/crates/opentelemetry), [`prometheus`](https://crates.io/crates/prometheus), [`prometheus-client`](https://crates.io/crates/prometheus-client) or [`metrics`](https://crates.io/crates/metrics))
- [📍 Attach exemplars](exemplars) to connect metrics with traces
- ⚡ Minimal runtime overhead

See [autometrics.dev](https://docs.autometrics.dev/) for more details on the ideas behind autometrics.

## Example Axum App

Autometrics isn't tied to any web framework, but this shows how you can use the library in an [Axum](https://github.com/tokio-rs/axum) server.

```rust
use autometrics::{autometrics, prometheus_exporter};
use axum::{routing::*, Router, Server};

// Instrument your functions with metrics
#[autometrics]
pub async fn create_user() -> Result<(), ()> {
  Ok(())
}

// Export the metrics to Prometheus
#[tokio::main]
pub async fn main() {
  prometheus_exporter::init();

  let app = Router::new()
      .route("/users", post(create_user))
      .route(
          "/metrics",
          get(|| async { prometheus_exporter::encode_http_response() }),
      );
  Server::bind(&([127, 0, 0, 1], 0).into())
      .serve(app.into_make_service());
}
```

## Generated Metrics

Autometrics uses the following metrics:
- `function.calls.count` to track the request and error rate
- `function.calls.duration` to track the latency
- (optionally) `function.calls.concurrent` to track the number of [concurrent calls](#track_concurrency)
- `build_info` tracks your application's version and commit to help spot when bugs or latency were introduced

For each of the `function.` metrics, Autometrics attaches the following labels:
- `function` - the name of the function
- `module` - the module path of the function

For the function call counter, Autometrics attaches these additional labels:
- `result` - if the function returns a `Result`, this will either be `ok` or `error`
- `caller` - the name of the (Autometrics-instrumented) function that called the current function

### `build_info` metric labels

The `build_info` metric makes it easy to correlate production issues with the commit or version that may have introduced bugs or latency.

By default, it attaches the `version` label, but you can also set up your project so that it attaches Git-related labels as well:

| Label | Compile-Time Environment Variables | Default |
|---|---|---|
| `version` | `AUTOMETRICS_VERSION` or `CARGO_PKG_VERSION` | `CARGO_PKG_VERSION` (set by cargo by default) |
| `commit` | `AUTOMETRICS_COMMIT` or `VERGEN_GIT_COMMIT` | `""` |
| `branch` | `AUTOMETRICS_BRANCH` or `VERGEN_GIT_BRANCH` | `""` |

#### (Optional) Using `vergen` to set the Git details

You can use the [`vergen`](https://crates.io/crates/vergen) crate to expose the Git information to Autometrics, which will then attach the labels to the `build_info` metric.

```sh
cargo add vergen --features git,gitcl
```

```rust
// build.rs

pub fn main() {
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

pub fn main() {
  // Reload Rust analyzer after changing the Prometheus URL to regenerate the links
  let prometheus_url = "https://your-prometheus-url.example";
  println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
}
```

### Feature flags

- `prometheus-exporter` - exports a Prometheus metrics collector and exporter. This is compatible with any of the [Metrics backends](#metrics-backends) and uses `prometheus-client` by default if none are explicitly selected

#### Metrics backends

> **Note**
>
> If you are **not** using the `prometheus-exporter`, you must ensure that you are using the exact same version of the metrics library as `autometrics` (and it must come from `crates.io` rather than git or another source). If not, the autometrics metrics will not appear in your exported metrics.

- `opentelemetry`  - use the [opentelemetry](https://crates.io/crates/opentelemetry) crate for producing metrics.
- `metrics` - use the [metrics](https://crates.io/crates/metrics) crate for producing metrics
- `prometheus` - use the [prometheus](https://crates.io/crates/prometheus) crate for producing metrics
- `prometheus-client` - use the official [prometheus-client](https://crates.io/crates/prometheus-client) crate for producing metrics

#### Exemplars (for integrating metrics with traces)

See the [exemplars module docs](https://docs.rs/autometrics/latest/autometrics/exemplars/index.html) for details about these features. Currently only supported with the `prometheus-client` backend.

- `exemplars-tracing` - extract arbitrary fields from `tracing::Span`s
- `exemplars-tracing-opentelemetry` - extract the `trace_id` and `span_id` from the `opentelemetry::Context`, which is attached to `tracing::Span`s by the `tracing-opentelemetry` crate

#### Custom objective values

By default, Autometrics supports a fixed set of percentiles and latency thresholds for objectives. Use these features to enable custom values:

- `custom-objective-latency` - enable this to use custom latency thresholds. Note, however, that the custom latency **must** match one of the buckets configured for your histogram or the alerts will not work. This is not currently compatible with the `prometheus` or `prometheus-exporter` feature.
- `custom-objective-percentile` - enable this to use custom objective percentiles. Note, however, that using custom percentiles requires generating a different recording and alerting rules file using the CLI + Sloth (see [here](https://github.com/autometrics-dev/autometrics-rs/tree/main/autometrics-cli)).
