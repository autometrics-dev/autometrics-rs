<!-- This is used on crates.io -->

![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

Metrics are a powerful and cost-efficient tool for understanding the health and performance of your code in production. But it's hard to decide what metrics to track and even harder to write queries to understand the data.

Autometrics provides a macro that makes it trivial to instrument any function with the most useful metrics: request rate, error rate, and latency. It standardizes these metrics and then generates powerful Prometheus queries based on your function details to help you quickly identify and debug issues in production.

## Benefits

- [✨ `#[autometrics]`](https://docs.rs/autometrics/latest/autometrics/attr.autometrics.html) macro adds useful metrics to any function or `impl` block, without you thinking about what metrics to collect
- 💡 Generates powerful Prometheus queries to help quickly identify and debug issues in production
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- [📊 Grafana dashboards](https://github.com/autometrics-dev/autometrics-shared#dashboards) work without configuration to visualize the performance of functions & [SLOs](https://docs.rs/autometrics/latest/autometrics/objectives/index.html)
- 🔍 Correlates your code's version with metrics to help identify commits that introduced errors or latency
- 📏 Standardizes metrics across services and teams to improve debugging
- ⚖️ Function-level metrics provide useful granularity without exploding cardinality
- [⚡ Minimal runtime overhead](#benchmarks)

## Advanced Features

- [🚨 Define alerts](https://docs.rs/autometrics/latest/autometrics/objectives/index.html) using SLO best practices directly in your source code
- [📍 Attach exemplars](https://docs.rs/autometrics/latest/autometrics/exemplars/index.html) automatically to connect metrics with traces
- [⚙️ Configurable](https://docs.rs/autometrics/latest/autometrics/#metrics-backends) metric collection library ([`opentelemetry`](https://crates.io/crates/opentelemetry), [`prometheus`](https://crates.io/crates/prometheus), [`prometheus-client`](https://crates.io/crates/prometheus-client) or [`metrics`](https://crates.io/crates/metrics))

See [autometrics.dev](https://docs.autometrics.dev/) for more details on the ideas behind autometrics.

## Example Axum App

Autometrics isn't tied to any web framework, but this shows how you can use the library in an [Axum](https://github.com/tokio-rs/axum) server.

```rust,ignore
use std::error::Error;
use autometrics::{autometrics, prometheus_exporter};
use axum::{routing::*, Router};
use std::net::Ipv4Addr;
use tokio::net::TcpListener;

// Instrument your functions with metrics
#[autometrics]
pub async fn create_user() -> Result<(), ()> {
    Ok(())
}

// Export the metrics to Prometheus
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    prometheus_exporter::init();

    let app = Router::new()
        .route("/users", post(create_user))
        .route(
            "/metrics",
            get(|| async { prometheus_exporter::encode_http_response() }),
        );


    let listener = TcpListener::bind((Ipv4Addr::from([127, 0, 0, 1]), 0)).await?;
    axum::serve(listener, app).await?;
    Ok(())
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
