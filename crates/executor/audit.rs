use crate::error::ExecutorError;
use crate::statement::SqlStatement;
use uuid::Uuid;
use vellum_migration::{sha256_hex, Migration};

const SQL_DB_INFO: &str = "SELECT current_database()::text, current_user::text, inet_client_addr()::text";

const SQL_INSERT_RUN: &str = "
INSERT INTO vellum.vellum_runs (
    id,
    started_at,
    finished_at,
    mode,
    status,
    db_name,
    db_user,
    client_host,
    vellum_version
)
VALUES ($1, now(), NULL, $2, $3, $4, $5, $6, $7)
";

const SQL_UPDATE_RUN_STATUS: &str = "
UPDATE vellum.vellum_runs
SET status = $2,
    finished_at = now()
WHERE id = $1
";

const SQL_SELECT_MIGRATION_CHECKSUM: &str = "
SELECT checksum
FROM vellum.vellum_migrations
WHERE version = $1
";

const SQL_INSERT_MIGRATION: &str = "
INSERT INTO vellum.vellum_migrations (
    version,
    name,
    checksum,
    execution_time_ms,
    success,
    error_code,
    error_message,
    run_id
)
VALUES ($1, $2, $3, $4, $5, NULL, $6, $7)
RETURNING id
";

const SQL_UPDATE_MIGRATION_SUCCESS: &str = "
UPDATE vellum.vellum_migrations
SET execution_time_ms = $2,
    success = TRUE
WHERE id = $1
";

const SQL_INSERT_STATEMENT: &str = "
INSERT INTO vellum.vellum_statements (
    migration_id,
    ordinal,
    statement_hash,
    statement_kind,
    transactional,
    execution_time_ms,
    success,
    error_message
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
";

pub async fn insert_run(pool: &sqlx::PgPool, vellum_version: &str) -> Result<Uuid, ExecutorError> {
    let (db_name, db_user, client_host): (String, String, Option<String>) = sqlx::query_as(SQL_DB_INFO)
        .fetch_one(pool)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: "<uncreated>".to_string(),
            operation: "db_info".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    let run_id = Uuid::new_v4();

    sqlx::query(SQL_INSERT_RUN)
        .bind(run_id)
        .bind("apply")
        .bind("running")
        .bind(db_name)
        .bind(db_user)
        .bind(client_host)
        .bind(vellum_version)
        .execute(pool)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: run_id.to_string(),
            operation: "insert_run".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    Ok(run_id)
}

pub async fn mark_run_success(pool: &sqlx::PgPool, run_id: Uuid) -> Result<(), ExecutorError> {
    sqlx::query(SQL_UPDATE_RUN_STATUS)
        .bind(run_id)
        .bind("success")
        .execute(pool)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: run_id.to_string(),
            operation: "mark_run_success".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    Ok(())
}

pub async fn mark_run_failed(
    pool: &sqlx::PgPool,
    run_id: Uuid,
    original_error: &ExecutorError,
) -> Result<(), ExecutorError> {
    sqlx::query(SQL_UPDATE_RUN_STATUS)
        .bind(run_id)
        .bind("failed")
        .execute(pool)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: run_id.to_string(),
            operation: "mark_run_failed".to_string(),
            message: e.to_string(),
            original_error: Some(original_error.to_string()),
        })?;

    Ok(())
}

pub async fn get_applied_checksum(
    pool: &sqlx::PgPool,
    version: &str,
) -> Result<Option<String>, ExecutorError> {
    let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_MIGRATION_CHECKSUM)
        .bind(version)
        .fetch_optional(pool)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: "<unknown>".to_string(),
            operation: "select_migration_checksum".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    Ok(row.map(|r| r.0))
}

pub async fn insert_migration(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    run_id: Uuid,
    migration: &Migration,
) -> Result<i64, ExecutorError> {
    let version = migration.version.to_string();

    let migration_id: i64 = sqlx::query_scalar(SQL_INSERT_MIGRATION)
        .bind(version)
        .bind(&migration.name)
        .bind(&migration.checksum)
        .bind(0_i32)
        .bind(false)
        .bind(Option::<&str>::None)
        .bind(run_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: run_id.to_string(),
            operation: "insert_migration".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    Ok(migration_id)
}

pub async fn mark_migration_success(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    migration_id: i64,
    execution_time_ms: i32,
) -> Result<(), ExecutorError> {
    sqlx::query(SQL_UPDATE_MIGRATION_SUCCESS)
        .bind(migration_id)
        .bind(execution_time_ms)
        .execute(&mut **tx)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: "<unknown>".to_string(),
            operation: "mark_migration_success".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    Ok(())
}

pub async fn insert_statement(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    migration_id: i64,
    stmt: &SqlStatement,
    execution_time_ms: i32,
    success: bool,
    error_message: Option<&str>,
) -> Result<(), ExecutorError> {
    let statement_hash = sha256_hex(stmt.sql.as_bytes());
    let kind = super::statement::statement_kind(&stmt.sql);

    sqlx::query(SQL_INSERT_STATEMENT)
        .bind(migration_id)
        .bind(stmt.ordinal)
        .bind(statement_hash)
        .bind(kind)
        .bind(true)
        .bind(execution_time_ms)
        .bind(success)
        .bind(error_message)
        .execute(&mut **tx)
        .await
        .map_err(|e| ExecutorError::RunTrackingFailed {
            run_id: "<unknown>".to_string(),
            operation: "insert_statement".to_string(),
            message: e.to_string(),
            original_error: None,
        })?;

    Ok(())
}
