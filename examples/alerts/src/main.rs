use autometrics::autometrics;
use clap::Parser;

// If you use clap to parse your CLI arguments, you can
// add another subcommand that you would use to output
// the Prometheus rules to a file.
#[derive(Parser)]
enum Cli {
    /// Generate Prometheus recording and alerting rules
    ///
    /// ```shell
    /// cargo run -p example-alerts -- generate-alerts > rules.yaml
    /// ```
    ///
    /// Or:
    /// ```shell
    /// cargo run -p example-alerts -- generate-alerts --output rules.yaml
    /// ```
    GenerateAlerts(GenerateAlertsArgs),
}

#[derive(Parser)]
struct GenerateAlertsArgs {
    /// Optional name of file to output the YAML to
    /// (default is to print to stdout)
    #[clap(short, long)]
    output: Option<String>,
}

/// Example HTTP handler function with alerts/SLOs defined
#[autometrics(
    alerts(success_rate = 99.9%, latency(99.9% <= 250ms)),
)]
pub async fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

pub fn main() {
    let app = Cli::parse();

    match app {
        Cli::GenerateAlerts(args) => {
            let output = generate_alerts();
            if let Some(filename) = args.output {
                std::fs::write(filename, output).unwrap();
            } else {
                println!("{}\n", output);
            }
        }
    }
}
