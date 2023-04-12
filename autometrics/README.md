
![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

**A Rust macro that makes it easy to understand the error rate, response time, and production usage of any function in your code.**

Jump from your IDE to live Prometheus charts for each HTTP/RPC handler, database method, or other piece of application logic.

<video src="https://user-images.githubusercontent.com/3262610/220152261-2ad6ab2b-f951-4b51-8d6e-855fb71440a3.mp4" autoplay loop muted width="100%"></video>

## Features
- ✨ [`#[autometrics]`](https://docs.rs/autometrics/latest/autometrics/attr.autometrics.html) macro instruments any function or `impl` block to track the most useful metrics
- 💡 Writes Prometheus queries so you can understand the data generated without knowing PromQL
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- 📊 (Coming Soon!) Grafana dashboard showing the performance of all instrumented functions
- 🚨 Enable Prometheus alerts using SLO best practices from simple annotations in your code
- ⚙️ Configurable metric collection library (`opentelemetry`, `prometheus`, or `metrics`)
- ⚡ Minimal runtime overhead

See [Why Autometrics?](https://github.com/autometrics-dev#why-autometrics) for more details on the ideas behind autometrics.

## Examples

To see autometrics in action:

1. Install [prometheus](https://prometheus.io/download/) locally
2. Run the [complete example](./examples/full-api):
```shell
cargo run -p example-full-api
```
3. Hover over the [function names](./examples/full-api/src/routes.rs#L13) to see the generated query links
(like in the image above) and try clicking on them to go straight to that Prometheus chart.

See the other [examples](./examples/) for details on how to use the various features and integrations.

Or run the example in Gitpod:

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/autometrics-dev/autometrics-rs)


## Exporting Prometheus Metrics

Prometheus works by polling a specific HTTP endpoint on your server to collect the current state of all the metrics it has in memory.

### For projects not currently using Prometheus metrics

Autometrics includes optional functions to help collect and prepare metrics to be collected by Prometheus.

In your `Cargo.toml` file, enable the optional `prometheus-exporter` feature:

```toml
autometrics = { version = "*", features = ["prometheus-exporter"] }
```

Then, call the `global_metrics_exporter` function in your `main` function:
```rust
pub fn main() {
  let _exporter = autometrics::global_metrics_exporter();
  // ...
}
```

And create a route on your API (probably mounted under `/metrics`) that returns the following:
```rust
pub fn get_metrics() -> (http::StatusCode, String) {
  match autometrics::encode_global_metrics() {
    Ok(metrics) => (http::StatusCode::OK, metrics),
    Err(err) => (http::StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
  }
}
```

### For projects already using custom Prometheus metrics

Autometrics uses existing metrics libraries (see [below](#metrics-libraries)) to produce and collect metrics.

If you are already using one of these to collect and export metrics, simply configure autometrics to use the same library and the metrics it produces will be exported alongside yours. You do not need to use the Prometheus exporter functions this library provides and you do not need a separate endpoint for autometrics' metrics.


## Alerts / SLOs

Autometrics makes it easy to add Prometheus alerts using Service-Level Objectives (SLOs) to a function or group of functions.

This works using pre-defined [Prometheus alerting rules](https://github.com/autometrics-dev/autometrics-shared#prometheus-recording--alerting-rules). By default, most of the recording rules are dormaint. They are enabled by specific metric labels that can be automatically attached by autometrics.

To use autometrics SLOs and alerts, create one or multiple [`Objective`s](https://docs.rs/autometrics/latest/autometrics/objectives/struct.Objective.html) based on the function(s) success rate and/or latency, as shown below. The `Objective` can be passed as an argument to the `autometrics` macro to include the given function in that objective.

```rust
use autometrics::autometrics;
use autometrics::objectives::{Objective, ObjectiveLatency, ObjectivePercentile};

const API_SLO: Objective = Objective::new("api")
    .success_rate(ObjectivePercentile::P99_9)
    .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P99);

#[autometrics(objective = API_SLO)]
pub fn api_handler() {
  // ...
}
```

## Configuring Autometrics

### Custom Prometheus URL
By default, Autometrics creates Prometheus query links that point to `http://localhost:9090`.

You can configure a custom Prometheus URL using a build-time environment in your `build.rs` file:

```rust
// build.rs

fn main() {
  let prometheus_url = "https://your-prometheus-url.example";
  println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
}
```
When using Rust Analyzer, you may need to reload the workspace in order for URL changes to take effect.

Note that the Prometheus URL is only included in function documentation comments so changing it will have no impact on the final compiled binary.


### Feature flags

- `prometheus-exporter` - exports a Prometheus metrics collector and exporter (compatible with any of the Metrics Libraries)
- `custom-objective-latency` - by default, Autometrics only supports a fixed set of latency thresholds for objectives. Enable this to use custom latency thresholds. Note, however, that the custom latency must match one of the buckets configured for your histogram, meaning you will not be able to use the default Prometheus exporter. This is not currently compatible with the `prometheus` or `prometheus-exporter` feature.
- `custom-objective-percentile` by default, Autometrics only supports a fixed set of objective percentiles. Enable this to use a custom percentile. Note, however, that using custom percentiles requires generating a different recording and alerting rules file using the CLI + Sloth.

#### Metrics Libraries

Configure the crate that autometrics will use to produce metrics by using one of the following feature flags:

- `opentelemetry` (enabled by default) - use the [opentelemetry](https://crates.io/crates/opentelemetry) crate for producing metrics
- `metrics` - use the [metrics](https://crates.io/crates/metrics) crate for producing metrics
- `prometheus` - use the [prometheus](https://crates.io/crates/prometheus) crate for producing metrics
