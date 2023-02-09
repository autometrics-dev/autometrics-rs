# Autometrics Full API Example

This is a complete example of how to use autometrics with an API server built with `axum`, `clap`, `thiserror`, and `tokio`.

## Running the example

### Starting the server

**Note:** You will need [prometheus](https://prometheus.io/download/) installed locally for the full experience.

This will start the server, generate some fake traffic, and run a local Prometheus instance that is configured to scrape the metrics from the server's `/metrics` endpoint.

```shell
cargo run -p example-full-api serve
```

### Generating alerting rules

This will output the [Prometheus alerting rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/) for this example API.

In this example, the API has an autometrics-instrumented middleware function that is run on all requests. This function is the one that has alerts defined on it.


```shell
cargo run -p example-full-api generate-alerts > alerting_rules.yml
```
