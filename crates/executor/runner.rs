use crate::audit;
use crate::error::ExecutorError;
use crate::statement;
use crate::transaction;
use uuid::Uuid;
use vellum_migration::Migration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub run_id: Uuid,
    pub applied: usize,
    pub skipped: usize,
}

#[derive(Clone)]
pub struct Runner {
    pool: sqlx::PgPool,
    vellum_version: String,
}

impl Runner {
    pub fn new(pool: sqlx::PgPool, vellum_version: impl Into<String>) -> Self {
        Self {
            pool,
            vellum_version: vellum_version.into(),
        }
    }

    pub async fn run(&self, migrations: &[Migration]) -> Result<RunReport, ExecutorError> {
        let run_id = audit::insert_run(&self.pool, &self.vellum_version).await?;

        let mut applied = 0usize;
        let mut skipped = 0usize;

        for m in migrations {
            let version_str = m.version.to_string();
            let existing_checksum = audit::get_applied_checksum(&self.pool, &version_str).await?;

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

                let _ = audit::mark_run_failed(&self.pool, run_id, &err).await;
                return Err(err);
            }

            match execute_one(&self.pool, run_id, m).await {
                Ok(()) => {
                    applied += 1;
                }
                Err(err) => {
                    let _ = audit::mark_run_failed(&self.pool, run_id, &err).await;
                    return Err(err);
                }
            }
        }

        audit::mark_run_success(&self.pool, run_id).await?;

        Ok(RunReport {
            run_id,
            applied,
            skipped,
        })
    }
}

async fn execute_one(
    pool: &sqlx::PgPool,
    run_id: Uuid,
    migration: &Migration,
) -> Result<(), ExecutorError> {
    let migration_version = migration.version;
    let statements = statement::split_statements(&migration.sql);

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
