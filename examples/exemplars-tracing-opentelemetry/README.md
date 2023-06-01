# Tracing + OpenTelemetry Exemplars Example

This example demonstrates how Autometrics can create exemplars with the `trace_id` and `span_id` from the [`opentelemetry::Context`](https://docs.rs/opentelemetry/latest/opentelemetry/struct.Context.html), which is created by the [`tracing_opentelemetry::OpenTelemetryLayer`](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/struct.OpenTelemetryLayer.html) and propagated by the `tracing` library.

> **Note**
>
> Prometheus must be [specifically configured](https://prometheus.io/docs/prometheus/latest/feature_flags/#exemplars-storage) to enable the experimental exemplars feature.
