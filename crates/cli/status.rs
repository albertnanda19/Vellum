use crate::args::StatusArgs;
use crate::error::CliError;
use crate::output;
use std::collections::HashSet;
use vellum_migration::{discover_migrations, MigrationDiscoveryError};

pub async fn run(_args: &StatusArgs, database_url_override: Option<&str>) -> Result<(), CliError> {
    let database_url = resolve_database_url(database_url_override)?;

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|_| {
            CliError::user_error(
                "Failed to connect to database",
                "Check DATABASE_URL and verify the database is reachable.",
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

    output::line(format!("Database: {database_name}"));
    match last_applied {
        Some((version, _name)) => output::line(format!("Last migration: {version}")),
        None => output::line("Last migration: none"),
    }
    output::line(format!("Applied migrations: {applied_count}"));
    output::line(format!("Pending migrations: {pending}"));
    match last_run_status {
        Some(status) => output::line(format!("Last run status: {status}")),
        None => output::line("Last run status: none"),
    }

    if pending > 0 {
        output::line("Run `vellum migrate` to apply");
    }

    Ok(())
}

async fn select_database_name(pool: &sqlx::PgPool) -> Result<String, CliError> {
    let row: Result<(String,), sqlx::Error> =
        sqlx::query_as("SELECT current_database()::text").fetch_one(pool).await;

    match row {
        Ok(r) => Ok(r.0),
        Err(e) => Err(CliError::migration_failed(
            "Status query failed",
            format!("Database query failed: {}", e.to_string()),
        )),
    }
}

fn resolve_database_url(database_url_override: Option<&str>) -> Result<String, CliError> {
    match database_url_override {
        Some(v) if !v.trim().is_empty() => Ok(v.to_string()),
        _ => match std::env::var("DATABASE_URL") {
            Ok(v) if !v.trim().is_empty() => Ok(v),
            _ => Err(CliError::user_error(
                "DATABASE_URL is required",
                "Set DATABASE_URL or pass --database-url to the CLI.",
            )),
        },
    }
}

fn map_discovery_error(err: MigrationDiscoveryError) -> CliError {
    CliError::user_error(
        format!("Migration discovery failed: {err}"),
        "Ensure the 'migrations' directory exists and contains valid .sql migration files.",
    )
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
            return CliError::user_error(
                "Vellum schema is not initialized",
                "Run `vellum migrate` to initialize the schema.",
            );
        }
    }

    CliError::migration_failed(
        "Status query failed",
        format!("Database query failed: {msg}"),
    )
}
