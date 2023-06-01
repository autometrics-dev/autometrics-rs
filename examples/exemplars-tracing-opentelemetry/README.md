# OpenTelemetry + Tracing Exemplars Example

This example demonstrates how Autometrics can pick up the `trace_id` and `span_id` from the [`opentelemetry::Context`](https://docs.rs/opentelemetry/latest/opentelemetry/struct.Context.html) and attach them to the metrics as [exemplars](https://grafana.com/docs/grafana/latest/fundamentals/exemplars/).

> **Note**
>
> Prometheus must be [specifically configured](https://prometheus.io/docs/prometheus/latest/feature_flags/#exemplars-storage) to enable the experimental exemplars feature.
