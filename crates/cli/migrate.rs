use crate::args::MigrateArgs;
use crate::error::CliError;
use crate::output;
use std::collections::HashSet;
use vellum_executor::{ExecutionMode, ExecutorError, Runner};
use vellum_migration::{discover_migrations, MigrationDiscoveryError};

pub async fn run(
    args: &MigrateArgs,
    database_url_override: Option<&str>,
    vellum_version: &str,
) -> Result<(), CliError> {
    let database_url = resolve_database_url(database_url_override)?;

    let migrations_dir = std::path::Path::new("migrations");
    let migrations = discover_migrations(migrations_dir).map_err(map_discovery_error)?;

    let migrator = vellum_db::SqlxDatabaseMigrator::connect(&database_url)
        .await
        .map_err(|_| {
            CliError::user_error(
                "Failed to connect to database",
                "Check DATABASE_URL and verify the database is reachable.",
            )
        })?;

    vellum_core::bootstrap::apply_baseline(&migrator)
        .await
        .map_err(|_| {
            CliError::migration_failed(
                "Failed to initialize vellum schema",
                "Run `vellum migrate` again, and check database permissions if the problem persists.",
            )
        })?;

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|_| {
            CliError::user_error(
                "Failed to connect to database",
                "Check DATABASE_URL and verify the database is reachable.",
            )
        })?;

    let applied_versions = select_applied_versions(&pool).await?;
    let pending_migrations: Vec<_> = migrations
        .iter()
        .filter(|m| !applied_versions.contains(&m.version.to_string()))
        .collect();

    if !args.dry_run {
        output::line("Connected to database");
    }

    let runner = Runner::new(pool, database_url, vellum_version);

    let mode = if args.dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Apply
    };

    if args.dry_run {
        output::line("Dry-run mode enabled");
        output::line(format!("Validating {} migrations", pending_migrations.len()));
    }

    let _report = runner
        .run_with_mode(mode, &migrations)
        .await
        .map_err(map_executor_error)?;

    if args.dry_run {
        output::line("All migrations valid");
        output::line("No changes were applied");
        return Ok(());
    }

    output::line("Acquired migration lock");
    output::line(format!(
        "Applying migrations ({} pending)",
        pending_migrations.len()
    ));
    for m in pending_migrations {
        output::line(format!("Migration {} -> OK", migration_label(m)));
    }
    output::line("Migration completed successfully");

    Ok(())
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

fn map_executor_error(err: ExecutorError) -> CliError {
    match err {
        ExecutorError::MigrationLockUnavailable { .. } => {
            CliError::lock_unavailable(
                "Another migration process is currently running",
                "Wait for the other process to finish or investigate stuck locks.",
            )
        }
        ExecutorError::ChecksumMismatch { version, .. } => {
            CliError::migration_failed(
                format!("Migration checksum mismatch detected at version {version}"),
                "This usually means the migration file was modified after being applied.",
            )
        }
        ExecutorError::MigrationAlreadyApplied { version } => {
            CliError::migration_failed(
                format!("Migration {version} is already applied"),
                "Ensure your migrations directory matches the database state.",
            )
        }
        ExecutorError::StatementExecutionFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
            "Fix the migration SQL and re-run `vellum migrate`.",
        ),
        ExecutorError::TransactionBeginFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
            "Database transaction could not start. Check database availability and permissions, then re-run `vellum migrate`.",
        ),
        ExecutorError::TransactionCommitFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
            "Database transaction could not commit. Check database health and permissions, then re-run `vellum migrate`.",
        ),
        ExecutorError::TransactionRollbackFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
            "Database transaction rollback failed. Investigate database state and re-run `vellum migrate`.",
        ),
        ExecutorError::DryRunValidationError {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Dry-run validation failed at version {migration_version}"),
            "Fix the migration SQL and re-run `vellum migrate --dry-run`.",
        ),
        ExecutorError::DryRunFailed { message, .. } => CliError::migration_failed(
            "Dry-run validation failed",
            {
                let _ = message;
                "Fix the migration SQL and re-run `vellum migrate --dry-run`."
            },
        ),
        ExecutorError::DryRunTransactionError { message, .. } => CliError::migration_failed(
            "Dry-run validation failed",
            {
                let _ = message;
                "Fix the migration SQL and re-run `vellum migrate --dry-run`."
            },
        ),
        ExecutorError::RunTrackingFailed { message, .. } => {
            CliError::migration_failed(
                "Migration failed",
                {
                    let _ = message;
                    "Migration run tracking failed. Check database permissions and schema, then re-run `vellum migrate`."
                },
            )
        }
        ExecutorError::LockAcquireFailed { .. } => CliError::lock_unavailable(
            "Failed to acquire migration lock",
            "Wait for other migration processes to finish, then try again.",
        ),
        ExecutorError::LockReleaseFailed { .. } => CliError::migration_failed(
            "Migration failed",
            "Migration lock could not be released cleanly. Try again and check for stuck locks.",
        ),
        ExecutorError::StatementParsingFailed {
            migration_version,
            message,
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
            format!("Failed to parse migration SQL: {message}"),
        ),
    }
}

fn migration_label(m: &vellum_migration::Migration) -> String {
    m.filename
        .strip_suffix(".sql")
        .unwrap_or(&m.filename)
        .to_string()
}

async fn select_applied_versions(pool: &sqlx::PgPool) -> Result<HashSet<String>, CliError> {
    let rows: Result<Vec<(String,)>, sqlx::Error> =
        sqlx::query_as("SELECT version FROM vellum.vellum_migrations WHERE success = TRUE")
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

fn map_status_sql_error(err: sqlx::Error) -> CliError {
    let msg = err.to_string();
    if msg.contains("vellum.vellum_migrations") {
        if msg.contains("does not exist") || msg.contains("undefined_table") {
            return CliError::user_error(
                "Vellum schema is not initialized",
                "Run `vellum migrate` to initialize the schema.",
            );
        }
    }

    CliError::migration_failed(
        "Migration failed",
        {
            let _ = msg;
            "Database query failed. Check database connectivity and permissions, then re-run `vellum migrate`."
        },
    )
}
