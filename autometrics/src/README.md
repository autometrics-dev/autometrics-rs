<!-- This is used on docs.rs -->

![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

Metrics are a powerful and cost-efficient tool for understanding the health and performance of your code in production. But it's hard to decide what metrics to track and even harder to write queries to understand the data.

Autometrics provides a macro that makes it trivial to instrument any function with the most useful metrics: request rate, error rate, and latency. It standardizes these metrics and then generates powerful Prometheus queries based on your function details to help you quickly identify and debug issues in production.

# Benefits

- [✨ `#[autometrics]`](autometrics) macro adds useful metrics to any function or `impl` block, without you thinking about what metrics to collect
- 💡 Generates powerful Prometheus queries to help quickly identify and debug issues in production
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- [📊 Grafana dashboards](https://github.com/autometrics-dev/autometrics-shared#dashboards) work without configuration to visualize the performance of functions & [SLOs](objectives)
- 🔍 Correlates your code's version with metrics to help identify commits that introduced errors or latency
- 📏 Standardizes metrics across services and teams to improve debugging
- ⚖️ Function-level metrics provide useful granularity without exploding cardinality
- [⚡ Minimal runtime overhead](https://github.com/autometrics-dev/autometrics-rs#benchmarks)

# Advanced Features

- [🚨 Define alerts](objectives) using SLO best practices directly in your source code
- [📍 Attach exemplars](exemplars) automatically to connect metrics with traces
- [⚙️ Configurable](#metrics-backends) metric collection library ([`opentelemetry`](https://crates.io/crates/opentelemetry), [`prometheus`](https://crates.io/crates/prometheus), [`prometheus-client`](https://crates.io/crates/prometheus-client) or [`metrics`](https://crates.io/crates/metrics))

See [autometrics.dev](https://docs.autometrics.dev/) for more details on the ideas behind autometrics.

# Example Axum App

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

# Configuring Autometrics

Because Autometrics combines a macro and a library, and supports multiple underlying metrics libraries, different settings are configured in different places.

## `AutometricsSettings`

See [`settings`].

## `build.rs` File

### Including Git commit details in the metrics

Autometrics produces a `build_info` metric and writes queries that make it easy to correlate production issues with the commit or version that may have introduced bugs or latency (see [this blog post](https://fiberplane.com/blog/autometrics-rs-0-4-spot-commits-that-introduce-errors-or-slow-down-your-application) for details).

The `version` label is set automatically based on the version in your crate's `Cargo.toml` file.

You can set `commit` and `branch` labels using the `AUTOMETRICS_COMMIT` and `AUTOMETRICS_BRANCH` environment variables , or you can use the [`vergen`](https://crates.io/crates/vergen) crate to attach them automatically:

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

### Custom Prometheus URL

The Autometrics macro inserts Prometheus query links into function documentation. By default, the links point to `http://localhost:9090` but you can configure it to use a custom URL using a compile-time environment variable in your `build.rs` file:

```rust
// build.rs

pub fn main() {
  // Reload Rust analyzer after changing the Prometheus URL to regenerate the links
  let prometheus_url = "https://your-prometheus-url.example";
  println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
}
```

### Disabling documentation generation

If you do not want Autometrics to insert Prometheus query links into the function documentation, set the `AUTOMETRICS_DISABLE_DOCS` compile-time environment variable:

```rust
// build.rs

pub fn main() {
  println!("cargo:rustc-env=AUTOMETRICS_DISABLE_DOCS=1");
}
```

## Feature flags

### Exporting metrics

- `prometheus-exporter` - exports a Prometheus metrics collector and exporter. This is compatible with any of the [Metrics backends](#metrics-backends) and uses `prometheus-client` by default if none are explicitly selected

### Pushing metrics

Easily push collected metrics to a OpenTelemetry collector and compatible software.
Combine one of the transport feature flags together with your runtime feature flag:

**Transport feature flags**:

- `otel-push-exporter-http` - metrics sent over HTTP(s) using `hyper`
- `otel-push-exporter-grpc` - metrics sent over gRPC using `tonic`

**Runtime feature flags**:

- `otel-push-exporter-tokio` - tokio
- `otel-push-exporter-tokio-current-thread` - tokio with `flavor = "current_thread"`
- `otel-push-exporter-async-std` - async-std

If you require more customization than these offered feature flags, enable just
`otel-push-exporter` and follow the [example](https://github.com/autometrics-dev/autometrics-rs/tree/main/examples/opentelemetry-push-custom).

### Metrics backends

> If you are exporting metrics yourself rather than using the `prometheus-exporter`, you must ensure that you are using the exact same version of the metrics library as `autometrics` (and it must come from `crates.io` rather than git or another source). If not, the autometrics metrics will not appear in your exported metrics.

- `opentelemetry-0_20`  - use the [opentelemetry](https://crates.io/crates/opentelemetry) crate for producing metrics.
- `metrics-0_21` - use the [metrics](https://crates.io/crates/metrics) crate for producing metrics
- `prometheus-0_13` - use the [prometheus](https://crates.io/crates/prometheus) crate for producing metrics
- `prometheus-client-0_21` - use the official [prometheus-client](https://crates.io/crates/prometheus-client) crate for producing metrics

### Exemplars (for integrating metrics with traces)

See the [exemplars module docs](https://docs.rs/autometrics/latest/autometrics/exemplars/index.html) for details about these features. Currently only supported with the `prometheus-client` backend.

- `exemplars-tracing` - extract arbitrary fields from `tracing::Span`s
- `exemplars-tracing-opentelemetry-0_20` or `exemplars-tracing-opentelemetry-0_21` - extract the `trace_id` and `span_id` from the `opentelemetry::Context`, which is attached to `tracing::Span`s by the `tracing-opentelemetry` crate

### Custom objective values

By default, Autometrics supports a fixed set of percentiles and latency thresholds for [`objectives`]. Use these features to enable custom values:

- `custom-objective-latency` - enable this to use custom latency thresholds. Note, however, that the custom latency **must** match one of the buckets configured for your histogram or the queries, recording rules, and alerts will not work.
- `custom-objective-percentile` - enable this to use custom objective percentiles. Note, however, that using custom percentiles requires generating a different recording and alerting rules file using the CLI + Sloth (see [here](https://github.com/autometrics-dev/autometrics-rs/tree/main/autometrics-cli)).
