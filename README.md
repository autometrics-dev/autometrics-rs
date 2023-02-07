# Autometrics 📈✨

[![Documentation](https://docs.rs/autometrics/badge.svg)](https://docs.rs/autometrics)
[![Crates.io](https://img.shields.io/crates/v/autometrics.svg)](https://crates.io/crates/autometrics)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/kHtwcH8As9)

**Autometrics is a macro that makes it trivial to add useful metrics to any function in your codebase.**

Easily understand and debug your production system using automatically generated queries. Autometrics adds links to Prometheus charts directly into each function's doc comments. It can even generate Prometheus [alerting rules](#alerting) for you and soon it will support generating Grafana dashboards.

### 1️⃣ Add `#[autometrics]` to any function or `impl` block

```rust
#[autometrics]
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Json<User>, ApiError> {
  // ...
}

#[autometrics]
impl Database {
  async fn save_user(&self, user: User) -> Result<User, DbError> {
    // ...
  }
}
```

### 2️⃣ Hover over the function name to see the generated queries

<img src="./assets/vs-code-example.png" alt="VS Code Hover Example" width="500">

### 3️⃣ Click a query link to go directly to the Prometheus chart for that function

<img src="./assets/prometheus-chart.png" alt="Prometheus Chart" width="500">

### 4️⃣ Go back to shipping features 🚀

## See it in action

1. Install [prometheus](https://prometheus.io/download/) locally
2. Run the [axum example](./examples/axum.rs):
```
cargo run --features="prometheus-exporter" --example axum
```
3. Hover over the [function names](./examples/axum.rs#L21) to see the generated query links
(like in the image above) and try clicking on them to go straight to that Prometheus chart.

## Why Autometrics?

### Metrics today are hard to use

Metrics are a powerful and relatively inexpensive tool for understanding your system in production.

However, they are still hard to use. Developers need to:
- Think about what metrics to track and which metric type to use (counter, histogram... 😕)
- Figure out how to write PromQL or another query language to get some data 😖
- Verify that the data returned actually answers the right question 😫

### Simplifying code-level observability

Many modern observability tools promise to make life "easy for developers" by automatically instrumenting your code with an agent or eBPF. Others ingest tons of logs or traces -- and charge high fees for the processing and storage.

Most of these tools treat your system as a black box and use complex and pricey processing to build up a model of your system. This, however, means that you need to map their model onto your mental model of the system in order to navigate the mountains of data.

Autometrics takes the opposite approach. Instead of throwing away valuable context and then using compute power to recreate it, it starts inside your code. It enables you to understand your production system at one of the most fundamental levels: from the function.

### Standardizing function-level metrics

Functions are one of the most fundamental building blocks of code. Why not use them as the building block for observability?

A core part of Autometrics is the simple idea of using standard metric names and a consistent scheme for tagging/labeling metrics. The three metrics currently used are: `function.calls.count`, `function.calls.duration`, and `function.calls.concurrent`.

### Labeling metrics with useful, low-cardinality function details

The following labels are added automatically to all three of the metrics: `function` and `module`.

For the function call counter, the following labels are also added:

- `caller` - (see ["Tracing Lite"](#tracing-lite) below)
- `result` - either `ok` or `error` if the function returns a `Result`
- `ok` / `error` - see the next section

#### Static return type labels

If the concrete `Result` types implement `Into<&'static str>`, the that string will also be added as a label value under the key `ok` or `error`.

For example, you can have the variant names of your error enum included as labels:
```rust
use strum::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum MyError {
  SomethingBad(String),
  Unknown,
  ComplexType { message: String },
}
```
In the above example, functions that return `Result<_, MyError>` would have an additional label `error` added with the values `something_bad`, `unknown`, or `complex_type`.

This is more useful than tracking external errors like HTTP status codes because multiple logical errors might map to the same status code.

Autometrics only supports `&'static str`s as labels to avoid the footgun of attaching labels with too many possible values. The [Prometheus docs](https://prometheus.io/docs/practices/naming/#labels) explain why this is important in the following warning:

> CAUTION: Remember that every unique combination of key-value label pairs represents a new time series, which can dramatically increase the amount of data stored. Do not use labels to store dimensions with high cardinality (many different label values), such as user IDs, email addresses, or other unbounded sets of values.

### "Tracing Lite"

A slightly unusual idea baked into autometrics is that by tracking more granular metrics, you can debug some issues that we would traditionally need to turn to tracing for.

Autometrics can be added to any function in your codebase, from HTTP handlers down to database methods.

This means that if you are looking into a problem with a specific HTTP handler, you can browse through the metrics of the functions _called by the misbehaving function_.

Simply hover over the function names of the nested function calls in your IDE to look at their metrics. Or, you can directly open the chart of the request or error rate of all functions called by a specific function.

### More to come!

Stay tuned for automatically generated dashboards!

## Optional Features

### Exporting Prometheus Metrics

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
pub fn get_metrics() -> (StatusCode, String) {
  match autometrics::encode_global_metrics() {
    Ok(metrics) => (StatusCode::OK, metrics),
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
  }
}
```

### Alerting

Autometrics can generate [alerting rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/) for Prometheus.

In your `Cargo.toml` file, enable the optional `alerts` feature:

```toml
autometrics = { version = "*", features = ["alerts"] }
```

Then, pass the `alerts` argument to the `autometrics` macro **for 1-3 top-level functions**:
```rust
#[autometrics(alerts(success_rate = 99.9%, latency(99% <= 200ms)))]
pub async fn handle_http_requests(req: Request) -> Result<Response, Error> {
  // ...
}
```

Use the `generate_alerts` function to produce the Prometheus rules YAML file:
```rust
use autometrics::generate_alerts;

fn print_prometheus_alerts() {
  println!("{}", generate_alerts());
}
```

## Configuring

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
Note that when using Rust Analyzer, you may need to reload the workspace in order for URL changes to take effect.


### Feature flags

- `alerts` - generate Prometheus [alerting rules](#alerting) to notify you when a given function's error rate or latency is too high
- `prometheus-exporter` - exports a Prometheus metrics collector and exporter (compatible with any of the Metrics Libraries)

#### Metrics Libraries

Configure the crate that autometrics will use to produce metrics by using one of the following feature flags:

- `opentelemetry` (enabled by default) - use the [opentelemetry](https://crates.io/crates/opentelemetry) crate for producing metrics
- `metrics` - use the [metrics](https://crates.io/crates/metrics) crate for producing metrics
- `prometheus` - use the [prometheus](https://crates.io/crates/prometheus) crate for producing metrics
