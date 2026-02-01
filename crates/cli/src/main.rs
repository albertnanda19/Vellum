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

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), vellum_core::Error> {
    let cli = Cli::parse();

    let database_url = std::env::var("VELLUM_DATABASE_URL")
        .map_err(|_| vellum_core::Error::message("VELLUM_DATABASE_URL is required"))?;

    let migrations_dir = std::env::var("VELLUM_MIGRATIONS_DIR").unwrap_or_else(|_| "migrations".to_string());
    let migrations_dir = std::path::PathBuf::from(migrations_dir);

    let migrator = vellum_core::runtime::build_migrator(&database_url).await?;
    vellum_core::bootstrap::apply_baseline(&migrator).await?;

    let orchestrator = vellum_core::runtime::build_orchestrator();

    match cli.command {
        Command::Migrate => {
            let _ = vellum_core::commands::migrate(&orchestrator);
            let _ = vellum_core::runtime::run_migrations(
                &database_url,
                &migrations_dir,
                env!("CARGO_PKG_VERSION"),
            )
            .await?;
        }
        Command::DryRun => {
            let _ = vellum_core::commands::dry_run(&orchestrator);
        }
        Command::Status => {
            let _ = vellum_core::commands::status(&orchestrator);
        }
    }

    Ok(())
}
