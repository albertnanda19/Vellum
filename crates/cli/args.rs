use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vellum")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Migrate(MigrateArgs),
    Status(StatusArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct MigrateArgs {
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct StatusArgs {}
