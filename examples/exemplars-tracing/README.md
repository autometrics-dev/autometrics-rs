# Tracing Exemplars Example

This example demonstrates how Autometrics can pick up the `trace_id` from [`tracing::Span`](https://docs.rs/tracing/latest/tracing/struct.Span.html)s and attach them to the metrics as [exemplars](https://grafana.com/docs/grafana/latest/fundamentals/exemplars/).

> **Note**
>
> Prometheus must be [specifically configured](https://prometheus.io/docs/prometheus/latest/feature_flags/#exemplars-storage) to enable the experimental exemplars feature.
