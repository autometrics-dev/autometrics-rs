use clap::Parser;

mod sloth;

#[derive(Parser)]
#[command(name = "autometrics", about)]
enum Cli {
    /// Generate an SLO definition file for use with https://sloth.dev
    GenerateSlothFile(sloth::Arguments),
}

fn main() {
    match Cli::parse() {
        Cli::GenerateSlothFile(command) => command.run(),
    }
}
