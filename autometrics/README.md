![GitHub_headerImage](https://user-images.githubusercontent.com/3262610/221191767-73b8a8d9-9f8b-440e-8ab6-75cb3c82f2bc.png)

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

Autometrics is an open source framework that makes it easy to understand the health and performance of your code in production.

The Rust library provides a macro that makes it trivial to track the most useful metrics for any function: request rate, error rate, and lantency. It then generates Prometheus queries to help you understand the data collected and inserts links to the live charts directly into each function's doc comments.

Autometrics also provides Grafana dashboards to get an overview of instrumented functions and enables you to create powerful alerts based on Service-Level Objectives (SLOs) directly in your source code.

```rust
use autometrics::autometrics;

#[autometrics]
pub async fn create_user() {
  // Now this function will have metrics!
}
```

Here is a demo of jumping from function docs to live Prometheus charts:

<video src="https://user-images.githubusercontent.com/3262610/220152261-2ad6ab2b-f951-4b51-8d6e-855fb71440a3.mp4" autoplay loop muted width="100%"></video>


## Features

- ✨ [`#[autometrics]`](https://docs.rs/autometrics/latest/autometrics/attr.autometrics.html) macro instruments any function or `impl` block to track the most useful metrics
- 💡 Writes Prometheus queries so you can understand the data generated without knowing PromQL
- 🔗 Injects links to live Prometheus charts directly into each function's doc comments
- [🔍 Identify commits](#identifying-commits-that-introduced-problems) that introduced errors or increased latency
- [🚨 Define alerts](#alerts--slos) using SLO best practices directly in your source code
- [📊 Grafana dashboards](#dashboards) work out of the box to visualize the performance of instrumented functions & SLOs
- [⚙️ Configurable](#metrics-libraries) metric collection library (`opentelemetry`, `prometheus`, or `metrics`)
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

Prometheus works by polling an HTTP endpoint on your server to collect the current values of all the metrics it has in memory.

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

If you are already using one of these to collect metrics, simply configure autometrics to use the same library and the metrics it produces will be exported alongside yours. You do not need to use the Prometheus exporter functions this library provides and you do not need a separate endpoint for autometrics' metrics.

## Identifying commits that introduced problems

Autometrics makes it easy to identify if a specific version or commit introduced errors or increased latencies.

It uses a separate metric (`build_info`) to track the version and, optionally, git commit of your service. It then writes queries that group metrics by the `version` and `commit` labels so you can spot correlations between those and potential issues.

The `version` is collected from the `CARGO_PKG_VERSION` environment variable, which `cargo` sets by default. You can override this by setting the compile-time environment variable `AUTOMETRICS_VERSION`. This follows the method outlined in [Exposing the software version to Prometheus](https://www.robustperception.io/exposing-the-software-version-to-prometheus/).

To set the `commit`, you can either set the compile-time environment variable `AUTOMETRICS_COMMIT`, or have it set automatically using the [vergen](https://crates.io/crates/vergen) crate:

```toml
# Cargo.toml

[build-dependencies]
vergen = { version = "8.1", features = ["git", "gitoxide"] }
```

```rust
// build.rs
fn main() {
  vergen::EmitBuilder::builder()
      .git_sha(true)
      .emit()
      .expect("Unable to generate build info");
}
```

## Dashboards

Autometrics provides [Grafana dashboards](https://github.com/autometrics-dev/autometrics-shared#dashboards) that will work for any project instrumented with the library.

## Alerts / SLOs

Autometrics makes it easy to add Prometheus alerts using Service-Level Objectives (SLOs) to a function or group of functions.

This works using pre-defined [Prometheus alerting rules](https://github.com/autometrics-dev/autometrics-shared#prometheus-recording--alerting-rules), which can be loaded via the `rule_files` field in your Prometheus configuration. By default, most of the recording rules are dormant. They are enabled by specific metric labels that can be automatically attached by autometrics.

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

Once you've added objectives to your code, you can use the [Autometrics Service-Level Objectives(SLO) Dashboard](https://github.com/autometrics-dev/autometrics-shared#dashboards) to visualize the current status of your objective(s).

## Configuring Autometrics

### Custom Prometheus URL

Autometrics creates Prometheus query links that point to `http://localhost:9090` by default but you can configure it to use a custom URL using an environment variable in your `build.rs` file:

```rust
// build.rs

fn main() {
  let prometheus_url = "https://your-prometheus-url.example";
  println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
}
```

When using Rust Analyzer, you may need to reload the workspace in order for URL changes to take effect.

The Prometheus URL is only included in documentation comments so changing it will have no impact on the final compiled binary.

### Feature flags

- `prometheus-exporter` - exports a Prometheus metrics collector and exporter (compatible with any of the Metrics Libraries)
- `custom-objective-latency` - by default, Autometrics only supports a fixed set of latency thresholds for objectives. Enable this to use custom latency thresholds. Note, however, that the custom latency **must** match one of the buckets configured for your histogram or the alerts will not work. This is not currently compatible with the `prometheus` or `prometheus-exporter` feature.
- `custom-objective-percentile` by default, Autometrics only supports a fixed set of objective percentiles. Enable this to use a custom percentile. Note, however, that using custom percentiles requires generating a different recording and alerting rules file using the CLI + Sloth (see [here](../autometrics-cli/)).

#### Metrics Libraries

Configure the crate that autometrics will use to produce metrics by using one of the following feature flags:

- `opentelemetry` (enabled by default) - use the [opentelemetry](https://crates.io/crates/opentelemetry) crate for producing metrics
- `metrics` - use the [metrics](https://crates.io/crates/metrics) crate for producing metrics
- `prometheus` - use the [prometheus](https://crates.io/crates/prometheus) crate for producing metrics
