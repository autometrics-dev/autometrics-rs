# Autometrics Alerts Example

This example shows how to use autometrics to generate [Prometheus alerting rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/).

If you want to use autometrics alerts for your application, you'll want to:

1. Determine the most important 1-3 top-level functions you want to alert on for your service and how reliable you need those functions to be.
2. Add a subcommand to your binary (as shown in this example) to generate the alerting rules.
3. Configure your Prometheus instance to [load the rules](https://prometheus.io/docs/prometheus/latest/configuration/recording_rules/#configuring-rules).
4. Probably use [Alertmanager](https://prometheus.io/docs/alerting/latest/alertmanager/) to de-duplicate alerts.


## Running the example

```shell
cargo run -p example-alerts generate-alerts > alerting_rules.yml
```

Or:

```shell
cargo run -p example-alerts generate-alerts --output alerting_rules.yml
```
