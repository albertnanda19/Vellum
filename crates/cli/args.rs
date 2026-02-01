use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vellum",
    version,
    disable_help_subcommand = true,
    propagate_version = true
)]
pub struct Cli {
    #[arg(long, env = "VELLUM_DATABASE_URL", value_name = "URL", global = true)]
    pub database_url: Option<String>,

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
