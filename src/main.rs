use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = vellum_cli::Cli::parse();

    let result = match cli.command {
        vellum_cli::Command::Migrate(args) => vellum_cli::migrate::run(
            &args,
            cli.database_url.as_deref(),
            env!("CARGO_PKG_VERSION"),
        )
        .await,
        vellum_cli::Command::Status(args) => {
            vellum_cli::status::run(&args, cli.database_url.as_deref()).await
        }
    };

    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}
