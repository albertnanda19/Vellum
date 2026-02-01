use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vellum")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Migrate,
    DryRun,
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Migrate => vellum_core::commands::migrate(),
        Command::DryRun => vellum_core::commands::dry_run(),
        Command::Status => vellum_core::commands::status(),
    }
}
