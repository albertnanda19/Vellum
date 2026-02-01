use crate::args::StatusArgs;
use crate::error::CliError;
use crate::output;
use crate::style::Style;
use crate::ui::Ui;
use std::collections::HashSet;
use vellum_migration::{discover_migrations, MigrationDiscoveryError};

pub async fn run(_args: &StatusArgs, database_url_override: Option<&str>) -> Result<(), CliError> {
    let database_url = resolve_database_url(database_url_override)?;

    let style = Style::detect();
    let ui = Ui::new(style);

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|_| {
            CliError::user_error("Failed to connect to database")
                .with_reason("Database connection failed.")
                .with_action(
                    "Check DATABASE_URL (or pass --database-url) and verify the database is reachable.",
                )
        })?;

    let migrations_dir = std::path::Path::new("migrations");
    let local = discover_migrations(migrations_dir).map_err(map_discovery_error)?;

    let applied_versions = select_applied_versions(&pool).await?;

    let mut pending = 0usize;
    for m in &local {
        if !applied_versions.contains(&m.version.to_string()) {
            pending += 1;
        }
    }

    let applied_count = applied_versions.len();
    let last_applied = select_last_applied(&pool).await?;
    let last_run_status = select_last_run_status(&pool).await?;

    let database_name = select_database_name(&pool).await?;

    for line in ui.header("Vellum Status") {
        output::line(line);
    }
    output::line(ui.kv("Database", &database_name));
    output::line("");

    output::line(ui.kv("Applied migrations", &applied_count.to_string()));
    output::line(ui.kv("Pending migrations", &pending.to_string()));

    let last_migration = match &last_applied {
        Some((version, name)) => last_migration_label(&local, version, name),
        None => "none".to_string(),
    };
    output::line(ui.kv("Last migration", &last_migration));

    let last_status = last_run_status.unwrap_or_else(|| "none".to_string());
    output::line(ui.kv("Last run status", &last_status));
    output::line(ui.footer());

    if pending > 0 {
        output::line(ui.info_line("Run `vellum migrate` to apply pending migrations"));
    }

    Ok(())
}

async fn select_database_name(pool: &sqlx::PgPool) -> Result<String, CliError> {
    let row: Result<(String,), sqlx::Error> =
        sqlx::query_as("SELECT current_database()::text").fetch_one(pool).await;

    match row {
        Ok(r) => Ok(r.0),
        Err(_) => Err(CliError::migration_failed("Status query failed")
            .with_reason("Database query failed.")
            .with_action("Verify database connectivity and permissions, then try again.")),
    }
}

fn resolve_database_url(database_url_override: Option<&str>) -> Result<String, CliError> {
    match database_url_override {
        Some(v) if !v.trim().is_empty() => Ok(v.to_string()),
        _ => match std::env::var("VELLUM_DATABASE_URL") {
            Ok(v) if !v.trim().is_empty() => Ok(v),
            _ => match std::env::var("DATABASE_URL") {
                Ok(v) if !v.trim().is_empty() => Ok(v),
                _ => Err(CliError::user_error("Database URL is required").with_action(
                    "Set VELLUM_DATABASE_URL (or DATABASE_URL) or pass --database-url to the CLI.",
                )),
            },
        },
    }
}

fn map_discovery_error(err: MigrationDiscoveryError) -> CliError {
    CliError::user_error("Migration discovery failed")
        .with_reason(err.to_string())
        .with_action("Ensure the 'migrations' directory exists and contains valid .sql migration files.")
}

async fn select_applied_versions(pool: &sqlx::PgPool) -> Result<HashSet<String>, CliError> {
    let rows: Result<Vec<(String,)>, sqlx::Error> = sqlx::query_as(
        "SELECT version FROM vellum.vellum_migrations WHERE success = TRUE",
    )
    .fetch_all(pool)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => return Err(map_status_sql_error(e)),
    };

    let mut out = HashSet::with_capacity(rows.len());
    for (v,) in rows {
        out.insert(v);
    }

    Ok(out)
}

async fn select_last_applied(pool: &sqlx::PgPool) -> Result<Option<(String, String)>, CliError> {
    let row: Result<Option<(String, String)>, sqlx::Error> = sqlx::query_as(
        "SELECT version, name FROM vellum.vellum_migrations WHERE success = TRUE ORDER BY version::bigint DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await;

    match row {
        Ok(r) => Ok(r),
        Err(e) => Err(map_status_sql_error(e)),
    }
}

async fn select_last_run_status(pool: &sqlx::PgPool) -> Result<Option<String>, CliError> {
    let row: Result<Option<(String,)>, sqlx::Error> = sqlx::query_as(
        "SELECT status FROM vellum.vellum_runs ORDER BY started_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await;

    match row {
        Ok(r) => Ok(r.map(|t| t.0)),
        Err(e) => Err(map_status_sql_error(e)),
    }
}

fn map_status_sql_error(err: sqlx::Error) -> CliError {
    let msg = err.to_string();
    if msg.contains("vellum.vellum_migrations") || msg.contains("vellum.vellum_runs") {
        if msg.contains("does not exist") || msg.contains("undefined_table") {
            return CliError::user_error("Vellum schema is not initialized")
                .with_action("Run `vellum migrate` to initialize the schema.");
        }
    }

    CliError::migration_failed("Status query failed")
        .with_reason({
            let _ = msg;
            "Database query failed."
        })
        .with_action("Verify database connectivity and permissions, then try again.")
}

fn last_migration_label(
    local: &[vellum_migration::Migration],
    version: &str,
    name: &str,
) -> String {
    if let Ok(v) = version.parse::<i64>() {
        for m in local {
            if m.version == v {
                return m
                    .filename
                    .strip_suffix(".sql")
                    .unwrap_or(&m.filename)
                    .to_string();
            }
        }
    }

    if name.trim().is_empty() {
        version.to_string()
    } else {
        format!("{version}_{name}")
    }
}
