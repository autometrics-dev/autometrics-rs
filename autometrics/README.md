<!-- This is used on crates.io -->

![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

Autometrics provides a macro that makes it easy to instrument any function with the most useful metrics: request rate, error rate, and latency. It then uses the instrumented function names to generate powerful Prometheus queries to help you identify and debug issues in production.

## Features

- ✨ [`#[autometrics]`](https://docs.rs/autometrics/latest/autometrics/attr.autometrics.html) macro instruments any function or `impl` block to track the [most useful metrics](https://github.com/autometrics-dev/autometrics-shared/blob/main/SPEC.md#metrics)
- 💡 Writes Prometheus queries so you can understand the data generated without knowing PromQL
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- [🔍 Identify commits](https://docs.rs/autometrics/latest/autometrics/#identifying-faulty-commits-with-the-build_info-metric) that introduced errors or increased latency
- [🚨 Define alerts](https://docs.rs/autometrics/latest/autometrics/objectives/index.html) using SLO best practices directly in your source code
- [📊 Grafana dashboards](https://github.com/autometrics-dev/autometrics-shared#dashboards) work out of the box to visualize the performance of instrumented functions & SLOs
- [⚙️ Configurable](https://docs.rs/autometrics/latest/autometrics/#metrics-backends) metric collection library ([`opentelemetry`](https://crates.io/crates/opentelemetry), [`prometheus`](https://crates.io/crates/prometheus), [`prometheus-client`](https://crates.io/crates/prometheus-client) or [`metrics`](https://crates.io/crates/metrics))
- [📍 Attach exemplars](https://docs.rs/autometrics/latest/autometrics/exemplars/index.html) to connect metrics with traces
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

## Quickstart

See the [Github repo README](https://github.com/autometrics-dev/autometrics-rs#quickstart) to quickly add `autometrics` to your project.

## Contributing

Issues, feature suggestions, and pull requests are very welcome!

If you are interested in getting involved:
- Join the conversation on [Discord](https://discord.gg/9eqGEs56UB)
- Ask questions and share ideas in the [Github Discussions](https://github.com/orgs/autometrics-dev/discussions)
- Take a look at the overall [Autometrics Project Roadmap](https://github.com/orgs/autometrics-dev/projects/1)
