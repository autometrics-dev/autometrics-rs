# Autometrics :chart_with_upwards_trend: :sparkles:
> Easily add metrics to your system -- and actually understand them using automatically customized Prometheus queries.

Metrics are powerful and relatively inexpensive, but they are still hard to use. Developers need to:
- Think about what metrics we want to track and which metric type to use (counter, gauge... :sleep:)
- Write PromQL or other query languages to get the data
- Verify that the data we get actually answers your question

Autometrics makes it easy to add metrics to any function in your codebase.
Then, it helps you understand the data by automatically writing common Prometheus for each function.
Explore your production metrics directly from your editor/IDE.

### :one: Add `#[autometrics]` to your code

```rust
#[autometrics]
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Json<User>, ApiError> {
  // ...
}
```

### :two: Hover over the function name to see the generated queries

<img src="./assets/vs-code-example.png" alt="VS Code Hover Example" width="500">

### :three: Click a query link to go directly to the Prometheus chart for that function

<img src="./assets/prometheus-chart.png" alt="Prometheus Chart" width="500">

### :four: Go back to shipping features :rocket:

## See it in action

1. Install [prometheus](https://prometheus.io/download/) locally
2. Run the [axum example](./examples/axum.rs):
```
cargo run --features="prometheus-exporter" --example axum
```
3. Hover over the [function names](./examples/axum.rs#L21) to see the generated query links
(like in the image above) and try clicking on them to go straight to that Prometheus chart.

## How it works

The `autometrics` macro rewrites your functions to include a variety of useful metrics.
It adds a counter for tracking function calls and errors (for functions that return `Result`s),
a histogram for latency, and a gauge for concurrent requests.

We currently use the [`opentelemetry`](https://crates.io/crates/opentelemetry) crate for producing metrics
in the [OpenTelemetry](https://opentelemetry.io/) format. This can be converted to the Prometheus export format, as well
as others, using different [exporters](https://github.com/open-telemetry/opentelemetry-rust#related-crates).

Autometrics can generate the PromQL queries and Prometheus links for each function because it is creating
the metrics using specific names and labeling conventions.

## API

### `#[autometrics]` Macro

For most use cases, you can simply add the `#[autometrics]` attribute to any function you want to collect metrics for. We recommend using it for any important function in your code base (HTTP handlers, database calls, etc), possibly excluding simple utilities that are infallible or have negligible execution time.

### Result Type Labels

By default, the metrics generated will have labels for the `function`, `module`, and `result` (where the value is `ok` or `error` if the function returns a `Result`).

The concrete result type(s) (the `T` and `E` in `Result<T, E>`) can also be included as labels if the types implement `Into<&'static str>`.

For example, if you have an `Error` enum to define specific error types, you can have the enum variant names included as labels:
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

#### Why no dynamic labels?

Autometrics only supports `&'static str`s as labels to avoid the footgun of attaching labels with too many possible values. The [Prometheus docs](https://prometheus.io/docs/practices/naming/#labels) explain why this is important in the following warning:

> CAUTION: Remember that every unique combination of key-value label pairs represents a new time series, which can dramatically increase the amount of data stored. Do not use labels to store dimensions with high cardinality (many different label values), such as user IDs, email addresses, or other unbounded sets of values.

## Exporting Prometheus Metrics

Autometrics includes optional functions to help collect and prepare metrics to be collected by Prometheus.

In your `Cargo.toml` file, enable the optional `prometheus-exporter` feature:

```toml
autometrics = { git = "ssh://git@github.com/fiberplane/autometrics-rs.git", branch = "main", features = ["prometheus-exporter"] }
```

Then, call the `global_metrics_exporter` function in your `main` function:
```rust
pub fn main() {
  let _exporter = global_metrics_exporter();
  // ...
}
```

And create a route on your API (probably mounted under `/metrics`) that returns the following:
```rust
pub fn get_metrics() -> (StatusCode, String) {
  match encode_global_metrics() {
    Ok(metrics) => (StatusCode::OK, metrics),
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
  }
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
