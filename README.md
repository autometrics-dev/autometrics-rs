# Autometrics
> Understand your system easily using automatically generated metrics and pre-built Prometheus queries.

Autometrics provides a macro you can use to instrument functions throughout your code base.
It creates metrics for you and then offers customized Prometheus queries for you to run to observe your system in production.

Autometrics currently generates the following queries for each instrumented function:
- Request rate
- Error rate
- Latency (95th and 99th percentiles)

## Example

```rust
use autometrics::autometrics;

/// Example HTTP handler function
#[autometrics]
pub async fn get_index_handler(db: Database, request: Request<Body>) -> Result<String, ()> {
  let foo = db.load_something_important().await;
  Ok("It worked!".to_string())
}
```

If you hovered over the `get_index_handler` definition in VS Code with Rust Analyzer installed, you would see:

<img src="./assets/vs-code-example.png" alt="VS Code Hover Example" height="200">

And clicking each of the metric links would take you straight to the Prometheus chart for that specific function.

## Configuring

### Custom Prometheus URL
By default, Autometrics creates Prometheus query links that point to `http://localhost:9090`.

You can configure a custom Prometheus URL by adding the following to your `build.rs` file:

```rust
let prometheus_url = "https://your-prometheus-url.example";
println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
```

For example:
```rust
// build.rs

fn main() {
  let prometheus_url = "https://your-prometheus-url.example";
  println!("cargo:rustc-env=PROMETHEUS_URL={prometheus_url}");
}
```
Note that when using Rust Analyzer, you'll need to reload the workspace in order for the changed URL to take effect.
