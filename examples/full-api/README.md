# Autometrics Full API Example

This is a complete example of how to use autometrics with an API server built with `axum`, `thiserror`, and `tokio`.

## Running the example

**Note:** You will need [prometheus](https://prometheus.io/download/) installed locally for the full experience.

This will start the server, generate some fake traffic, and run a local Prometheus instance that is configured to scrape the metrics from the server's `/metrics` endpoint.

```shell
cargo run -p example-full-api
```
