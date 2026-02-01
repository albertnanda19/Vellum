use crate::error::LockError;
use sqlx::PgConnection;

pub const VELLUM_LOCK_KEY_NAMESPACE: u64 = 0x5645_4c4c_554d_4c4b;

pub fn lock_key(database_name: &str) -> i64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in database_name.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let mixed = hash ^ VELLUM_LOCK_KEY_NAMESPACE;
    mixed as i64
}

pub async fn current_database(conn: &mut PgConnection) -> Result<String, LockError> {
    let row: (String,) = sqlx::query_as("select current_database()")
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| LockError::LockAcquireFailed {
            message: format!("current_database query failed: {e}"),
        })?;

    Ok(row.0)
}

pub async fn try_lock(conn: &mut PgConnection, key: i64) -> Result<bool, LockError> {
    let row: (bool,) = sqlx::query_as("select pg_try_advisory_lock($1)")
        .bind(key)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| LockError::LockAcquireFailed {
            message: format!("pg_try_advisory_lock failed: {e}"),
        })?;

    Ok(row.0)
}

pub async fn unlock(conn: &mut PgConnection, key: i64) -> Result<bool, LockError> {
    let row: (bool,) = sqlx::query_as("select pg_advisory_unlock($1)")
        .bind(key)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| LockError::LockReleaseFailed {
            message: format!("pg_advisory_unlock failed: {e}"),
        })?;

    Ok(row.0)
}
