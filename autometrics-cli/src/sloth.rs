use clap::Parser;
use std::{fs::write, path::PathBuf};

#[derive(Parser)]
pub struct Arguments {
    /// The objective percentages to support.
    ///
    /// Note that the objective used in autometrics-instrumented code must match
    /// one of these values in order for the alert to work.
    #[clap(long, default_values = &["90", "95", "99", "99.9"])]
    objectives: Vec<String>,

    /// Minimum traffic to trigger alerts, specified as events/minute.
    ///
    /// Alerts will only trigger for an objective if the total call-rate of functions
    /// comprising the objective is greather than this threshold.
    ///
    /// Defaults to "at least 1 event per minute"
    ///
    /// Note that the total of calls is made on matching _both_ the "name"
    /// attribute and the percentile targets; e.g. a function from an "API, 90%"
    /// objective and one from an "API, 99%" objective count for 2 separate
    /// low-traffic threshold.
    #[clap(short, long, default_value_t = 1.0)]
    alerting_traffic_threshold: f64,

    /// Output path where the SLO file should be written.
    ///
    /// If not specified, the SLO file will be printed to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,
}

impl Arguments {
    pub fn run(&self) {
        let sloth_file =
            generate_sloth_file(&self.objectives, self.alerting_traffic_threshold / 60.0);
        if let Some(output_path) = &self.output {
            write(output_path, sloth_file)
                .unwrap_or_else(|err| panic!("Error writing SLO file to {output_path:?}: {err}"));
        } else {
            println!("{}", sloth_file);
        }
    }
}

fn generate_sloth_file(objectives: &[impl AsRef<str>], min_calls_per_second: f64) -> String {
    let mut sloth_file = "version: prometheus/v1
service: autometrics
slos:
"
    .to_string();

    for objective in objectives {
        sloth_file.push_str(&generate_success_rate_slo(
            objective.as_ref(),
            min_calls_per_second,
        ));
    }
    for objective in objectives {
        sloth_file.push_str(&generate_latency_slo(
            objective.as_ref(),
            min_calls_per_second,
        ));
    }

    sloth_file
}

fn generate_success_rate_slo(objective_percentile: &str, min_calls_per_second: f64) -> String {
    let objective_percentile_no_decimal = objective_percentile.replace('.', "_");

    format!("  - name: success-rate-{objective_percentile_no_decimal}
    objective: {objective_percentile}
    description: Common SLO based on function success rates
    sli:
      events:
        error_query: sum by (objective_name, objective_percentile, service_name) (rate({{__name__=~\"function_calls(_count)?(_total)?\",objective_percentile=\"{objective_percentile}\",result=\"error\"}}[{{{{.window}}}}]))
        total_query: sum by (objective_name, objective_percentile, service_name) (rate({{__name__=~\"function_calls(_count)?(_total)?\",objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}])) >= {min_calls_per_second}
    alerting:
      name: High Error Rate SLO - {objective_percentile}%
      labels:
        category: success-rate
      annotations:
        summary: \"High error rate on the `{{{{$labels.objective_name}}}}` SLO for the `{{{{$labels.service_name}}}}` service\"
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}

fn generate_latency_slo(objective_percentile: &str, min_calls_per_second: f64) -> String {
    let objective_percentile_no_decimal = objective_percentile.replace('.', "_");

    format!("  - name: latency-{objective_percentile_no_decimal}
    objective: {objective_percentile}
    description: Common SLO based on function latency
    sli:
      events:
        error_query: >
          sum by (objective_name, objective_percentile, service_name) (rate(function_calls_duration_count{{objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]))
          -
          (sum by (objective_name, objective_percentile, service_name) (
            label_join(rate({{__name__=~\"function_calls_duration(_seconds)?_bucket\", objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"objective_latency_threshold\")
            and
            label_join(rate({{__name__=~\"function_calls_duration(_seconds)?_bucket\", objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"le\")
          ))
        total_query: sum by (objective_name, objective_percentile, service_name) (rate({{__name__=~\"function_calls_duration(_seconds)?_count\", objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}])) >= {min_calls_per_second}
    alerting:
      name: High Latency SLO - {objective_percentile}%
      labels:
        category: latency
      annotations:
        summary: \"High latency on the `{{{{$labels.objective_name}}}}` SLO for the `{{{{$labels.service_name}}}}` service\"
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}
