use crate::args::StatusArgs;
use crate::error::CliError;
use std::collections::HashSet;
use vellum_migration::{discover_migrations, MigrationDiscoveryError};

pub async fn run(_args: &StatusArgs) -> Result<(), CliError> {
    let database_url = database_url()?;

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|e| CliError::message(format!("Database connection failed: {e}")))?;

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

    match last_applied {
        Some((version, name)) => {
            println!("last_applied: {version} {name}");
        }
        None => {
            println!("last_applied: none");
        }
    }

    println!("applied: {applied_count}");
    println!("pending: {pending}");

    match last_run_status {
        Some(status) => println!("last_run_status: {status}"),
        None => println!("last_run_status: none"),
    }

    Ok(())
}

fn database_url() -> Result<String, CliError> {
    match std::env::var("DATABASE_URL") {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(CliError::message("DATABASE_URL is required")),
    }
}

fn map_discovery_error(err: MigrationDiscoveryError) -> CliError {
    CliError::message(format!("Migration discovery failed: {err}"))
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
            return CliError::message("Vellum schema is not initialized; run `vellum migrate` first");
        }
    }

    CliError::message(format!("Status query failed: {msg}"))
}
