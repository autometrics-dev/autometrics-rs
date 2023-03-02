use clap::Parser;
use rust_decimal::Decimal;
use std::{fs::write, path::PathBuf};

#[derive(Parser)]
pub struct Arguments {
    /// The objective percentages to support.
    ///
    /// Note that the objective used in autometrics-instrumented code must match
    /// one of these values in order for the alert to work.
    #[clap(long, default_values = &["90", "95", "99", "99.9"])]
    objectives: Vec<Decimal>,

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

fn generate_sloth_file(objectives: &[Decimal]) -> String {
    let mut sloth_file = "version: prometheus/v1
service: autometrics
slos:
"
    .to_string();

    for objective in objectives {
        sloth_file.push_str(&generate_success_rate_slo(objective));
    }
    for objective in objectives {
        sloth_file.push_str(&generate_latency_slo(objective));
    }

    sloth_file
}

fn generate_success_rate_slo(objective: &Decimal) -> String {
    let objective_fraction = objective / Decimal::from(100);
    let objective_fraction_no_decimal = objective_fraction.to_string().replace(".", "");

    format!("  - name: success-rate-{objective_fraction_no_decimal}
    objective: {objective}
    description: Common SLO based on function success rates
    sli:
      events:
        error_query: sum by (slo_name, objective) (rate(function_calls_count{{objective=\"{objective}\",result=\"error\"}}[{{{{.window}}}}]))
        total_query: sum by (slo_name, objective) (rate(function_calls_count{{objective=\"{objective}\"}}[{{{{.window}}}}]))
    alerting:
      name: High Error Rate SLO - {objective}%
      labels:
        category: success-rate
      annotations:
        summary: \"High error rate on SLO: {{{{$labels.slo_name}}}}\"
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}

fn generate_latency_slo(objective: &Decimal) -> String {
    let objective_fraction = objective / Decimal::from(100);
    let objective_fraction_no_decimal = objective_fraction.to_string().replace(".", "");

    format!("  - name: latency-{objective_fraction_no_decimal}
    objective: {objective}
    description: Common SLO based on function latency
    sli:
      events:
        error_query: >
          sum by (slo_name, objective) (rate(function_calls_duration_bucket{{objective=\"{objective}\"}}[{{{{.window}}}}]))
          -
          (sum by (slo_name, objective) (
            label_join(rate(function_calls_duration_bucket{{objective=\"{objective}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"target_latency\")
            and
            label_join(rate(function_calls_duration_bucket{{objective=\"{objective}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"le\")
          ))
        total_query: sum by (slo_name, objective) (rate(function_calls_duration_bucket{{objective=\"{objective}\"}}[{{{{.window}}}}]))
    alerting:
      name: High Latency SLO - {objective}%
      labels:
        category: latency
      annotations:
        summary: \"High latency on SLO: {{{{$labels.slo_name}}}}\"
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}
