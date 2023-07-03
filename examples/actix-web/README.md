# Autometrics actix-web Example

This shows how you can instrument [`actix-web`](https://github.com/actix/actix-web)
HTTP handler functions with autometrics.

## Running the example

**Note:** You will need [Prometheus](https://prometheus.io/download/) installed locally for the full experience.

```shell
cargo run -p example-actix-web
```

This will start the server, generate some fake traffic, and run a local Prometheus instance that is configured
to scrape the metrics from the server's `/metrics` endpoint.
