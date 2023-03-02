use clap::Parser;

mod sloth;

#[derive(Parser)]
enum Cli {
    /// Generate an SLO definition file for use with https://sloth.dev
    GenerateSlothFile(sloth::Arguments),
}

fn main() {
    match Cli::parse() {
        Cli::GenerateSlothFile(command) => command.run(),
    }
}
