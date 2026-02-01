use crate::audit;
use crate::error::ExecutorError;
use crate::statement;
use vellum_migration::Migration;

pub async fn run(
    pool: &sqlx::PgPool,
    vellum_version: &str,
    migrations: &[Migration],
) -> Result<crate::runner::RunReport, ExecutorError> {
    let run_id = audit::insert_run_with_mode(pool, "dry_run", vellum_version)
        .await
        .map_err(|e| ExecutorError::DryRunFailed {
            message: "run tracking insert failed".to_string(),
            original_error: e.to_string(),
        })?;

    let planned = match plan_migrations(pool, migrations).await {
        Ok(planned) => planned,
        Err(err) => {
            let _ = audit::mark_run_failed(pool, run_id, &err).await;
            return Err(err);
        }
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            let err = ExecutorError::DryRunTransactionError {
                operation: "begin".to_string(),
                message: e.to_string(),
                original_error: None,
            };
            let _ = audit::mark_run_failed(pool, run_id, &err).await;
            return Err(err);
        }
    };

    for m in &planned.to_apply {
        let statements = statement::split_statements(&m.sql);

        for stmt in &statements {
            if let Err(err) = statement::execute_statement(&mut tx, m.version, stmt).await {
                let mapped = map_validation_error(m.version, Some(stmt.ordinal), Some(&stmt.sql), &err);

                let rollback_res = tx.rollback().await;
                if let Err(rollback_err) = rollback_res {
                    let rollback_mapped = ExecutorError::DryRunTransactionError {
                        operation: "rollback_after_failure".to_string(),
                        message: rollback_err.to_string(),
                        original_error: Some(mapped.to_string()),
                    };

                    let _ = audit::mark_run_failed(pool, run_id, &rollback_mapped).await;
                    return Err(rollback_mapped);
                }

                let _ = audit::mark_run_failed(pool, run_id, &mapped).await;
                return Err(mapped);
            }
        }
    }

    match tx.rollback().await {
        Ok(()) => {}
        Err(e) => {
            let err = ExecutorError::DryRunTransactionError {
                operation: "rollback".to_string(),
                message: e.to_string(),
                original_error: None,
            };
            let _ = audit::mark_run_failed(pool, run_id, &err).await;
            return Err(err);
        }
    }

    audit::mark_run_success(pool, run_id)
        .await
        .map_err(|e| ExecutorError::DryRunFailed {
            message: "run tracking mark success failed".to_string(),
            original_error: e.to_string(),
        })?;

    Ok(crate::runner::RunReport {
        run_id,
        applied: planned.to_apply.len(),
        skipped: planned.skipped,
    })
}

struct PlannedMigrations<'a> {
    to_apply: Vec<&'a Migration>,
    skipped: usize,
}

async fn plan_migrations<'a>(
    pool: &sqlx::PgPool,
    migrations: &'a [Migration],
) -> Result<PlannedMigrations<'a>, ExecutorError> {
    let mut to_apply = Vec::new();
    let mut skipped = 0usize;

    for m in migrations {
        let version_str = m.version.to_string();
        let existing_checksum = audit::get_applied_checksum(pool, &version_str)
            .await
            .map_err(|e| ExecutorError::DryRunFailed {
                message: "applied checksum lookup failed".to_string(),
                original_error: e.to_string(),
            })?;

        if let Some(db_checksum) = existing_checksum {
            if db_checksum == m.checksum {
                skipped += 1;
                continue;
            }

            return Err(ExecutorError::DryRunValidationError {
                migration_version: m.version,
                statement_ordinal: None,
                sql_snippet: None,
                message: format!(
                    "checksum mismatch for version {} (db={}, fs={})",
                    m.version, db_checksum, m.checksum
                ),
            });
        }

        to_apply.push(m);
    }

    Ok(PlannedMigrations { to_apply, skipped })
}

fn map_validation_error(
    migration_version: i64,
    statement_ordinal: Option<i32>,
    sql: Option<&str>,
    err: &ExecutorError,
) -> ExecutorError {
    ExecutorError::DryRunValidationError {
        migration_version,
        statement_ordinal,
        sql_snippet: sql.map(sql_snippet),
        message: err.to_string(),
    }
}

fn sql_snippet(sql: &str) -> String {
    const MAX_CHARS: usize = 200;
    let trimmed = sql.trim();

    let mut out = String::with_capacity(trimmed.len().min(MAX_CHARS) + 1);
    for (i, ch) in trimmed.chars().enumerate() {
        if i >= MAX_CHARS {
            out.push('…');
            break;
        }

        match ch {
            '\n' | '\r' | '\t' => out.push(' '),
            _ => out.push(ch),
        }
    }

    out
}
