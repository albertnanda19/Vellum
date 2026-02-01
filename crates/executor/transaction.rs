use crate::error::ExecutorError;

pub async fn begin<'a>(
    pool: &'a sqlx::PgPool,
    migration_version: i64,
) -> Result<sqlx::Transaction<'a, sqlx::Postgres>, ExecutorError> {
    pool.begin().await.map_err(|e| ExecutorError::TransactionBeginFailed {
        migration_version,
        message: e.to_string(),
    })
}

pub async fn commit(
    tx: sqlx::Transaction<'_, sqlx::Postgres>,
    migration_version: i64,
) -> Result<(), ExecutorError> {
    tx.commit()
        .await
        .map_err(|e| ExecutorError::TransactionCommitFailed {
            migration_version,
            message: e.to_string(),
        })
}

pub async fn rollback(
    tx: sqlx::Transaction<'_, sqlx::Postgres>,
    migration_version: i64,
    original_error: &ExecutorError,
) -> Result<(), ExecutorError> {
    tx.rollback()
        .await
        .map_err(|e| ExecutorError::TransactionRollbackFailed {
            migration_version,
            message: e.to_string(),
            original_error: original_error.to_string(),
        })
}
