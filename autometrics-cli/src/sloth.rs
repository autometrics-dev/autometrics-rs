use clap::Parser;
use rust_decimal::Decimal;
use std::{fs::write, path::PathBuf};

#[derive(Parser)]
pub struct Arguments {
    /// The number of SLOs to support.
    ///
    /// Note that the actual number of SLOs from Sloth's perspective will be double
    /// this because we generate both success rate and latency SLOs.
    #[clap(long, default_value = "3")]
    num_slos: u8,

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
        let sloth_file = generate_sloth_file(self.num_slos, &self.objectives);
        if let Some(output_path) = &self.output {
            write(output_path, sloth_file)
                .expect(&format!("Error writing SLO file to {:?}", output_path));
        } else {
            println!("{}", sloth_file);
        }
    }
}

fn generate_sloth_file(num_slos: u8, objectives: &[Decimal]) -> String {
    let mut sloth_file = "version: prometheus/v1
service: autometrics
slos:
"
    .to_string();

    for slo_number in 1..=num_slos {
        for objective in objectives {
            sloth_file.push_str(&generate_success_rate_slo(slo_number, objective));
        }
    }
    for slo_number in 1..=num_slos {
        for objective in objectives {
            sloth_file.push_str(&generate_latency_slo(slo_number, objective));
        }
    }

    sloth_file
}

fn generate_success_rate_slo(slo_number: u8, objective: &Decimal) -> String {
    let objective_fraction = objective / Decimal::from(100);
    let objective_fraction_no_decimal = objective_fraction.to_string().replace(".", "");
    format!("  - name: success-rate-{slo_number}-{objective_fraction_no_decimal}
    objective: {objective}
    description: Common SLO based on function success rates
    labels:
      objective: {objective_fraction}
      slo: {slo_number}
    sli:
      events:
        error_query: sum(rate(function_calls_count{{slo=\"{slo_number}\",objective=\"{objective}\",result=\"error\"}}[{{{{.window}}}}]))
        total_query: sum(rate(function_calls_count{{slo=\"{slo_number}\",objective=\"{objective}\"}}[{{{{.window}}}}]))
    alerting:
      name: High Error Rate SLO {slo_number} - {objective}%
      labels:
        category: success-rate
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}

fn generate_latency_slo(slo_number: u8, objective: &Decimal) -> String {
    let objective_fraction = objective / Decimal::from(100);
    let objective_fraction_no_decimal = objective_fraction.to_string().replace(".", "");

    format!("  - name: latency-{slo_number}-{objective_fraction_no_decimal}
    objective: {objective}
    description: Common SLO based on function latency
    labels:
      objective: {objective_fraction}
      slo: {slo_number}
    sli:
      events:
        error_query: >
          sum(rate(function_calls_duration_bucket{{slo=\"{slo_number}\",objective=\"{objective_fraction}\"}}[{{{{.window}}}}]))
          -
          (sum(
            label_join(rate(function_calls_duration_bucket{{slo=\"{slo_number}\",objective=\"{objective_fraction}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"target_latency\")
            and
            label_join(rate(function_calls_duration_bucket{{slo=\"{slo_number}\",objective=\"{objective_fraction}\"}}[{{{{.window}}}}]), \"autometrics_check_label_equality\", \"\", \"le\")
          ))
        total_query: sum(rate(function_calls_duration_bucket{{slo=\"{slo_number}\",objective=\"{objective_fraction}\"}}[{{{{.window}}}}]))
    alerting:
      name: High Latency SLO {slo_number} - {objective}%
      labels:
        category: success-rate
      page_alert:
        labels:
          severity: page
      ticket_alert:
        labels:
          severity: ticket
")
}
