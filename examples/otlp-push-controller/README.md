# Autometrics + OTLP push controller

This example demonstrates how you can push autometrics via OTLP gRPC protocol to an OTEL-Collector or another compatible solution.

## ⚠️ Warning

At this step, it's absolutely required that the version of the opentelemetry crates used in the project matches the version used as a dependency in `autometrics-rs` crate (See [Issue 91](https://github.com/autometrics-dev/autometrics-rs/issues/91)).

## Running the example

### Start a basic OTEL-Collector

You can use the `otel-collector-config.yaml` file to start an otel-collector container that listen on 0.0.0.0:4317 for incoming otlp-gRPC traffic, and export received metrics to standard output.

```bash
docker run -d --name otel-col \
    -p 4317:4317 -p 13133:13133 \
    -v $PWD/otel-collector-config.yaml:/etc/otelcol/config.yaml \
    otel/opentelemetry-collector:latest
```

### Execute example code

```shell
cargo run -p example-otlp-push-controller
```

## OpenTelemetry Metrics Push Controller

The metric push controller is implemented as from this [example](https://github.com/open-telemetry/opentelemetry-rust/blob/f20c9b40547ee20b6ec99414bb21abdd3a54d99b/examples/basic-otlp/src/main.rs#L35-L52) from `opentelemetry-rust` crate.
