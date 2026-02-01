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

    let orchestrator = vellum_core::runtime::build_orchestrator();

    match cli.command {
        Command::Migrate => {
            let _ = vellum_core::commands::migrate(&orchestrator);
        }
        Command::DryRun => {
            let _ = vellum_core::commands::dry_run(&orchestrator);
        }
        Command::Status => {
            let _ = vellum_core::commands::status(&orchestrator);
        }
    }
}
