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

    /// Output path where the SLO file should be written.
    ///
    /// If not specified, the SLO file will be printed to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,
}

impl Arguments {
    pub fn run(&self) {
        let sloth_file = generate_sloth_file(&self.objectives);
        if let Some(output_path) = &self.output {
            write(output_path, sloth_file)
                .expect(&format!("Error writing SLO file to {:?}", output_path));
        } else {
            println!("{}", sloth_file);
        }
    }
}

fn generate_sloth_file(objectives: &[impl AsRef<str>]) -> String {
    let mut sloth_file = "version: prometheus/v1
service: autometrics
slos:
"
    .to_string();

    for objective in objectives {
        sloth_file.push_str(&generate_success_rate_slo(objective.as_ref()));
    }
    for objective in objectives {
        sloth_file.push_str(&generate_latency_slo(objective.as_ref()));
    }

    sloth_file
}

fn generate_success_rate_slo(objective_percentile: &str) -> String {
    let objective_percentile_no_decimal = objective_percentile.replace(".", "_");

    format!("  - name: success-rate-{objective_percentile_no_decimal}
    objective: {objective_percentile}
    description: Common SLO based on function success rates
    sli:
      events:
        error_query: sum by (objective_name, objective_percentile) (rate(function_calls_count{{objective_percentile=\"{objective_percentile}\",result=\"error\"}}[{{{{.window}}}}]))
        total_query: sum by (objective_name, objective_percentile) (rate(function_calls_count{{objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]))
    alerting:
      name: High Error Rate SLO - {objective_percentile}%
      labels:
        category: success-rate
      annotations:
        summary: \"High error rate on SLO: {{{{$labels.objective_name}}}}\"
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}

fn generate_latency_slo(objective_percentile: &str) -> String {
    let objective_percentile_no_decimal = objective_percentile.replace(".", "_");

    format!("  - name: latency-{objective_percentile_no_decimal}
    objective: {objective_percentile}
    description: Common SLO based on function latency
    sli:
      events:
        error_query: >
          sum by (objective_name, objective_percentile) (rate(function_calls_duration_bucket{{objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]))
          -
          (sum by (objective_name, objective_percentile) (
            label_join(rate(function_calls_duration_bucket{{objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"objective_latency_threshold\")
            and
            label_join(rate(function_calls_duration_bucket{{objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"le\")
          ))
        total_query: sum by (objective_name, objective_percentile) (rate(function_calls_duration_bucket{{objective_percentile=\"{objective_percentile}\"}}[{{{{.window}}}}]))
    alerting:
      name: High Latency SLO - {objective_percentile}%
      labels:
        category: latency
      annotations:
        summary: \"High latency on SLO: {{{{$labels.objective_name}}}}\"
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}
