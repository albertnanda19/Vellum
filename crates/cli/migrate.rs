use crate::args::MigrateArgs;
use crate::error::CliError;
use vellum_executor::{ExecutionMode, ExecutorError, Runner};
use vellum_migration::{discover_migrations, MigrationDiscoveryError};

pub async fn run(args: &MigrateArgs, vellum_version: &str) -> Result<(), CliError> {
    let database_url = database_url()?;

    let migrations_dir = std::path::Path::new("migrations");
    let migrations = discover_migrations(migrations_dir).map_err(map_discovery_error)?;

    let migrator = vellum_db::SqlxDatabaseMigrator::connect(&database_url)
        .await
        .map_err(|e| CliError::message(format!("Database connection failed: {e}")))?;

    vellum_core::bootstrap::apply_baseline(&migrator)
        .await
        .map_err(|e| CliError::message(format!("Failed to initialize vellum schema: {e}")))?;

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|e| CliError::message(format!("Database connection failed: {e}")))?;

    let runner = Runner::new(pool, database_url, vellum_version);

    let mode = if args.dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Apply
    };

    let report = runner
        .run_with_mode(mode, &migrations)
        .await
        .map_err(map_executor_error)?;

    println!("run_id: {}", report.run_id);
    println!("mode: {}", mode.as_str());
    println!("applied: {}", report.applied);
    println!("skipped: {}", report.skipped);

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

fn map_executor_error(err: ExecutorError) -> CliError {
    match err {
        ExecutorError::MigrationLockUnavailable { .. } => {
            CliError::message("Another migration process is currently running")
        }
        ExecutorError::ChecksumMismatch { version, .. } => {
            CliError::message(format!("Migration failed at version {version} (checksum mismatch)"))
        }
        ExecutorError::MigrationAlreadyApplied { version } => {
            CliError::message(format!("Migration {version} is already applied"))
        }
        ExecutorError::StatementExecutionFailed {
            migration_version,
            statement_ordinal,
            message,
            ..
        } => CliError::message(format!(
            "Migration failed at version {migration_version} (statement {statement_ordinal}): {message}"
        )),
        ExecutorError::TransactionBeginFailed {
            migration_version,
            message,
        } => CliError::message(format!(
            "Migration failed at version {migration_version} (transaction begin failed): {message}"
        )),
        ExecutorError::TransactionCommitFailed {
            migration_version,
            message,
        } => CliError::message(format!(
            "Migration failed at version {migration_version} (transaction commit failed): {message}"
        )),
        ExecutorError::TransactionRollbackFailed {
            migration_version,
            message,
            ..
        } => CliError::message(format!(
            "Migration failed at version {migration_version} (transaction rollback failed): {message}"
        )),
        ExecutorError::DryRunValidationError {
            migration_version,
            statement_ordinal,
            message,
            ..
        } => {
            let ordinal = match statement_ordinal {
                Some(o) => o.to_string(),
                None => "<unknown>".to_string(),
            };
            CliError::message(format!(
                "Dry-run failed at version {migration_version} (statement {ordinal}): {message}"
            ))
        }
        ExecutorError::DryRunFailed { message, .. } => CliError::message(format!("Dry-run failed: {message}")),
        ExecutorError::DryRunTransactionError {
            operation, message, ..
        } => CliError::message(format!("Dry-run failed ({operation}): {message}")),
        ExecutorError::RunTrackingFailed { message, .. } => {
            CliError::message(format!("Migration failed (run tracking): {message}"))
        }
        ExecutorError::LockAcquireFailed { .. } => {
            CliError::message("Failed to acquire migration lock")
        }
        ExecutorError::LockReleaseFailed { .. } => {
            CliError::message("Failed to release migration lock")
        }
    }
}
