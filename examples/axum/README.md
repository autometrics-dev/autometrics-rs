# Autometrics Axum Example

This shows how you can instrument [Axum](https://github.com/tokio-rs/axum) HTTP handler functions with autometrics.

For a more complete example also using axum, see the [full API example](../full-api/).

## Running the example

```shell
cargo run -p example-axum
```

This will start the server, generate some fake traffic, and run a local Prometheus instance that is configured to scrape the metrics from the server's `/metrics` endpoint.
