use crate::advisory;
use crate::error::LockError;
use core::time::Duration;
use sqlx::{Connection, PgConnection};
use tokio::time::{sleep, Instant};

pub struct AdvisoryLockGuard {
    conn: Option<PgConnection>,
    key: i64,
}

impl AdvisoryLockGuard {
    pub async fn acquire(database_url: &str, timeout: Duration) -> Result<Self, LockError> {
        let mut conn = PgConnection::connect(database_url)
            .await
            .map_err(|e| LockError::LockAcquireFailed {
                message: format!("connect failed: {e}"),
            })?;

        let db_name = advisory::current_database(&mut conn).await?;
        let key = advisory::lock_key(&db_name);

        let deadline = Instant::now() + timeout;
        let poll = Duration::from_millis(200);

        loop {
            if advisory::try_lock(&mut conn, key).await? {
                return Ok(Self {
                    conn: Some(conn),
                    key,
                });
            }

            if Instant::now() >= deadline {
                return Err(LockError::MigrationLockUnavailable {
                    timeout_ms: timeout.as_millis() as u64,
                });
            }

            sleep(poll).await;
        }
    }

    pub async fn release(mut self) -> Result<(), LockError> {
        let mut conn = match self.conn.take() {
            Some(conn) => conn,
            None => {
                return Err(LockError::LockReleaseFailed {
                    message: "lock connection missing".to_string(),
                })
            }
        };

        match advisory::unlock(&mut conn, self.key).await {
            Ok(true) => conn.close().await.map_err(|e| LockError::LockReleaseFailed {
                message: format!("connection close failed: {e}"),
            }),
            Ok(false) => {
                let close_res = conn.close().await;
                let close_msg = match close_res {
                    Ok(()) => "".to_string(),
                    Err(e) => format!("; close_error={e}"),
                };

                Err(LockError::LockReleaseFailed {
                    message: format!("pg_advisory_unlock returned false{close_msg}"),
                })
            }
            Err(err) => {
                let close_res = conn.close().await;
                let close_msg = match close_res {
                    Ok(()) => "".to_string(),
                    Err(e) => format!("; close_error={e}"),
                };

                Err(LockError::LockReleaseFailed {
                    message: format!("{err}{close_msg}"),
                })
            }
        }
    }
}
