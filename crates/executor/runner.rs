use crate::audit;
use crate::error::ExecutorError;
use crate::mode::ExecutionMode;
use crate::dry_run;
use crate::statement;
use crate::transaction;
use core::time::Duration;
use uuid::Uuid;
use vellum_lock::{AdvisoryLockGuard, LockError};
use vellum_migration::Migration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub run_id: Uuid,
    pub applied: usize,
    pub skipped: usize,
}

async fn run_apply(
    pool: &sqlx::PgPool,
    vellum_version: &str,
    migrations: &[Migration],
) -> Result<RunReport, ExecutorError> {
    let run_id = audit::insert_run(pool, vellum_version).await?;

    let mut applied = 0usize;
    let mut skipped = 0usize;

    for m in migrations {
        let version_str = m.version.to_string();
        let existing_checksum = audit::get_applied_checksum(pool, &version_str).await?;

        if let Some(db_checksum) = existing_checksum {
            if db_checksum == m.checksum {
                skipped += 1;
                continue;
            }

            let err = ExecutorError::ChecksumMismatch {
                version: m.version,
                expected: db_checksum,
                actual: m.checksum.clone(),
            };

            let _ = audit::mark_run_failed(pool, run_id, &err).await;
            return Err(err);
        }

        match execute_one(pool, run_id, m).await {
            Ok(()) => {
                applied += 1;
            }
            Err(err) => {
                let _ = audit::mark_run_failed(pool, run_id, &err).await;
                return Err(err);
            }
        }
    }

    audit::mark_run_success(pool, run_id).await?;

    Ok(RunReport {
        run_id,
        applied,
        skipped,
    })
}

#[derive(Clone)]
pub struct Runner {
    pool: sqlx::PgPool,
    vellum_version: String,
    database_url: String,
}

impl Runner {
    pub fn new(
        pool: sqlx::PgPool,
        database_url: impl Into<String>,
        vellum_version: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            database_url: database_url.into(),
            vellum_version: vellum_version.into(),
        }
    }

    pub async fn run(&self, migrations: &[Migration]) -> Result<RunReport, ExecutorError> {
        self.run_with_mode(ExecutionMode::Apply, migrations).await
    }

    pub async fn run_with_mode(
        &self,
        mode: ExecutionMode,
        migrations: &[Migration],
    ) -> Result<RunReport, ExecutorError> {
        let lock_timeout = Duration::from_secs(30);
        let lock = AdvisoryLockGuard::acquire(&self.database_url, lock_timeout)
            .await
            .map_err(map_lock_error)?;

        let result = self.run_locked(mode, migrations).await;
        match lock.release().await {
            Ok(()) => result,
            Err(release_err) => {
                let original_error = result.as_ref().err().map(|e| e.to_string());
                Err(ExecutorError::LockReleaseFailed {
                    message: release_err.to_string(),
                    original_error,
                })
            }
        }
    }

    async fn run_locked(
        &self,
        mode: ExecutionMode,
        migrations: &[Migration],
    ) -> Result<RunReport, ExecutorError> {
        match mode {
            ExecutionMode::Apply => run_apply(&self.pool, &self.vellum_version, migrations).await,
            ExecutionMode::DryRun => {
                dry_run::run(&self.pool, &self.vellum_version, migrations).await
            }
        }
    }
}

fn map_lock_error(err: LockError) -> ExecutorError {
    match err {
        LockError::MigrationLockUnavailable { timeout_ms } => {
            ExecutorError::MigrationLockUnavailable { timeout_ms }
        }
        LockError::LockAcquireFailed { message } => ExecutorError::LockAcquireFailed { message },
        LockError::LockReleaseFailed { message } => ExecutorError::LockReleaseFailed {
            message,
            original_error: None,
        },
    }
}

async fn execute_one(
    pool: &sqlx::PgPool,
    run_id: Uuid,
    migration: &Migration,
) -> Result<(), ExecutorError> {
    let migration_version = migration.version;
    let statements = statement::split_statements(
        &migration.sql,
        Some(&migration.filename),
        migration_version,
    )?;

    let mut tx = transaction::begin(pool, migration_version).await?;

    let migration_id = audit::insert_migration(&mut tx, run_id, migration).await?;
    let migration_started = std::time::Instant::now();

    for stmt in &statements {
        match statement::execute_statement(&mut tx, migration_version, stmt).await {
            Ok(execution_time_ms) => {
                audit::insert_statement(&mut tx, migration_id, stmt, execution_time_ms, true, None)
                    .await?;
            }
            Err(err) => {
                let execution_time_ms = match &err {
                    ExecutorError::StatementExecutionFailed {
                        execution_time_ms,
                        ..
                    } => *execution_time_ms,
                    _ => 0,
                };

                audit::insert_statement(
                    &mut tx,
                    migration_id,
                    stmt,
                    execution_time_ms,
                    false,
                    Some(&err.to_string()),
                )
                .await?;

                let _ = transaction::rollback(tx, migration_version, &err).await;
                return Err(err);
            }
        }
    }

    let migration_elapsed_ms = statement::duration_ms(migration_started.elapsed());
    audit::mark_migration_success(&mut tx, migration_id, migration_elapsed_ms).await?;
    transaction::commit(tx, migration_version).await?;

    Ok(())
}
