use crate::args::MigrateArgs;
use crate::error::CliError;
use crate::output;
use crate::style::Style;
use crate::ui::Ui;
use std::collections::{HashMap, HashSet};
use vellum_executor::{ExecutionMode, ExecutorError, Runner};
use vellum_migration::{discover_migrations, MigrationDiscoveryError};

pub async fn run(
    args: &MigrateArgs,
    database_url_override: Option<&str>,
    vellum_version: &str,
) -> Result<(), CliError> {
    let database_url = resolve_database_url(database_url_override)?;

    let style = Style::detect();
    let ui = Ui::new(style);

    let migrations_dir = std::path::Path::new("migrations");
    let migrations = discover_migrations(migrations_dir).map_err(map_discovery_error)?;

    let migrator = vellum_db::SqlxDatabaseMigrator::connect(&database_url)
        .await
        .map_err(|_| {
            CliError::user_error("Failed to connect to database")
                .with_reason("Database connection failed.")
                .with_action(
                    "Check DATABASE_URL (or pass --database-url) and verify the database is reachable.",
                )
        })?;

    vellum_core::bootstrap::apply_baseline(&migrator)
        .await
        .map_err(|_| {
            CliError::migration_failed("Failed to initialize vellum schema")
                .with_reason("Schema initialization failed.")
                .with_action(
                    "Run `vellum migrate` again, and check database permissions if the problem persists.",
                )
        })?;

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|_| {
            CliError::user_error("Failed to connect to database")
                .with_reason("Database connection failed.")
                .with_action("Check DATABASE_URL (or pass --database-url) and verify the database is reachable.")
        })?;
    let pool_for_queries = pool.clone();

    let database_name = select_database_name(&pool).await?;

    let applied_versions = select_applied_versions(&pool).await?;
    let pending_migrations: Vec<_> = migrations
        .iter()
        .filter(|m| !applied_versions.contains(&m.version.to_string()))
        .collect();

    if args.dry_run {
        for line in ui.header("Vellum Migration (dry-run)") {
            output::line(line);
        }
        output::line(ui.kv("Database", &database_name));
        output::line("");
        output::line(ui.ok_line("Connected to database"));
    } else {
        for line in ui.header("Vellum Migration") {
            output::line(line);
        }
        output::line(ui.kv("Database", &database_name));
        output::line(ui.kv("Mode", "apply"));
        output::line("");
        output::line(ui.ok_line("Connected to database"));
    }

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

    if args.dry_run {
        output::line(ui.ok_line("Advisory lock acquired"));
        output::line(ui.info_line(&format!(
            "Validating {} migrations",
            pending_migrations.len()
        )));
        output::line("");
        output::line(ui.ok_line("All migrations are valid"));
        output::line(ui.ok_line("No changes were applied"));
        output::line(ui.footer());
        return Ok(());
    }

    output::line(ui.ok_line("Advisory lock acquired"));
    output::line(ui.info_line(&format!(
        "Applying {} migrations",
        pending_migrations.len()
    )));
    output::line("");

    let run_id = report.run_id.to_string();
    let execution_times = match select_run_migration_times(&pool_for_queries, &run_id).await {
        Ok(m) => m,
        Err(_) => HashMap::new(),
    };

    for m in pending_migrations {
        let suffix_string;
        let suffix = match execution_times.get(&m.version) {
            Some(ms) => {
                suffix_string = format!("({}ms)", ms);
                Some(suffix_string.as_str())
            }
            None => None,
        };

        output::line(ui.list_item_with_suffix(&migration_label(m), "OK", suffix));
    }
    output::line("");
    output::line(ui.ok_line("Migration completed successfully"));
    output::line(ui.footer());

    Ok(())
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

fn map_executor_error(err: ExecutorError) -> CliError {
    match err {
        ExecutorError::MigrationLockUnavailable { .. } => {
            CliError::lock_unavailable("Another migration process is currently running")
                .with_action("Wait for the other process to finish or investigate stuck locks.")
        }
        ExecutorError::ChecksumMismatch { version, .. } => {
            CliError::migration_failed(format!("Migration failed at version {version}"))
                .with_reason("Checksum mismatch detected." )
                .with_meaning("The migration file was modified after being applied.")
                .with_action("Restore the original migration file or reset the database.")
        }
        ExecutorError::MigrationAlreadyApplied { version } => {
            CliError::migration_failed(format!("Migration failed at version {version}"))
                .with_reason("Migration is already applied." )
                .with_action("Ensure your migrations directory matches the database state.")
        }
        ExecutorError::StatementExecutionFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
        )
        .with_reason("Statement execution failed.")
        .with_action("Fix the migration SQL and re-run `vellum migrate`."),
        ExecutorError::TransactionBeginFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
        )
        .with_reason("Database transaction could not start.")
        .with_action("Check database availability and permissions, then re-run `vellum migrate`."),
        ExecutorError::TransactionCommitFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
        )
        .with_reason("Database transaction could not commit.")
        .with_action("Check database health and permissions, then re-run `vellum migrate`."),
        ExecutorError::TransactionRollbackFailed {
            migration_version,
            ..
        } => CliError::migration_failed(
            format!("Migration failed at version {migration_version}"),
        )
        .with_reason("Database transaction rollback failed.")
        .with_action("Investigate database state and re-run `vellum migrate`."),
        ExecutorError::DryRunValidationError {
            migration_version,
            ..
        } => CliError::migration_failed(format!(
            "Migration failed at version {migration_version}"
        ))
        .with_reason("Dry-run validation failed.")
        .with_action("Fix the migration SQL and re-run `vellum migrate --dry-run`."),
        ExecutorError::DryRunFailed { message, .. } => CliError::migration_failed(
            "Dry-run validation failed",
        )
        .with_reason({
            let _ = message;
            "Dry-run execution failed."
        })
        .with_action("Fix the migration SQL and re-run `vellum migrate --dry-run`."),
        ExecutorError::DryRunTransactionError { message, .. } => CliError::migration_failed(
            "Dry-run validation failed",
        )
        .with_reason({
            let _ = message;
            "Dry-run transaction error."
        })
        .with_action("Fix the migration SQL and re-run `vellum migrate --dry-run`."),
        ExecutorError::RunTrackingFailed { message, .. } => {
            CliError::migration_failed("Migration failed")
                .with_reason({
                    let _ = message;
                    "Migration run tracking failed."
                })
                .with_action("Check database permissions and schema, then re-run `vellum migrate`.")
        }
        ExecutorError::LockAcquireFailed { .. } => CliError::lock_unavailable(
            "Failed to acquire migration lock",
        )
        .with_action("Wait for other migration processes to finish, then try again."),
        ExecutorError::LockReleaseFailed { .. } => CliError::migration_failed(
            "Migration failed",
        )
        .with_reason("Migration lock could not be released cleanly.")
        .with_action("Try again and check for stuck locks."),
        ExecutorError::StatementParsingFailed {
            migration_version,
            message,
        } => CliError::migration_failed(format!("Migration failed at version {migration_version}"))
            .with_reason("Failed to parse migration SQL.")
            .with_action({
                let _ = message;
                "Fix the migration SQL and re-run `vellum migrate`."
            }),
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
            return CliError::user_error("Vellum schema is not initialized")
                .with_action("Run `vellum migrate` to initialize the schema.");
        }
    }

    CliError::migration_failed("Migration failed")
        .with_reason({
            let _ = msg;
            "Database query failed."
        })
        .with_action("Check database connectivity and permissions, then re-run `vellum migrate`.")
}

async fn select_database_name(pool: &sqlx::PgPool) -> Result<String, CliError> {
    let row: Result<(String,), sqlx::Error> =
        sqlx::query_as("SELECT current_database()::text").fetch_one(pool).await;

    match row {
        Ok(r) => Ok(r.0),
        Err(_) => Err(CliError::migration_failed("Migration failed")
            .with_reason("Status query failed.")
            .with_action("Verify database connectivity and permissions, then try again.")),
    }
}

async fn select_run_migration_times(
    pool: &sqlx::PgPool,
    run_id: &str,
) -> Result<HashMap<i64, i32>, CliError> {
    let rows: Result<Vec<(String, i32)>, sqlx::Error> = sqlx::query_as(
        "SELECT version, execution_time_ms FROM vellum.vellum_migrations WHERE run_id = $1::uuid AND success = TRUE",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => return Err(map_status_sql_error(e)),
    };

    let mut out = HashMap::with_capacity(rows.len());
    for (version, ms) in rows {
        if let Ok(v) = version.parse::<i64>() {
            out.insert(v, ms);
        }
    }

    Ok(out)
}
