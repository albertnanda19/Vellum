use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorError {
    MigrationLockUnavailable {
        timeout_ms: u64,
    },
    LockAcquireFailed {
        message: String,
    },
    LockReleaseFailed {
        message: String,
        original_error: Option<String>,
    },
    MigrationAlreadyApplied {
        version: i64,
    },
    ChecksumMismatch {
        version: i64,
        expected: String,
        actual: String,
    },
    StatementExecutionFailed {
        migration_version: i64,
        statement_ordinal: i32,
        execution_time_ms: i32,
        statement: String,
        message: String,
    },
    TransactionCommitFailed {
        migration_version: i64,
        message: String,
    },
    RunTrackingFailed {
        run_id: String,
        operation: String,
        message: String,
        original_error: Option<String>,
    },
    TransactionBeginFailed {
        migration_version: i64,
        message: String,
    },
    TransactionRollbackFailed {
        migration_version: i64,
        message: String,
        original_error: String,
    },
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutorError::MigrationLockUnavailable { timeout_ms } => {
                write!(f, "migration lock unavailable (timeout_ms={timeout_ms})")
            }
            ExecutorError::LockAcquireFailed { message } => {
                write!(f, "lock acquire failed: {message}")
            }
            ExecutorError::LockReleaseFailed {
                message,
                original_error,
            } => {
                if let Some(original_error) = original_error {
                    write!(f, "lock release failed: {message}; original_error={original_error}")
                } else {
                    write!(f, "lock release failed: {message}")
                }
            }
            ExecutorError::MigrationAlreadyApplied { version } => {
                write!(f, "migration already applied: version={version}")
            }
            ExecutorError::ChecksumMismatch {
                version,
                expected,
                actual,
            } => write!(
                f,
                "checksum mismatch for version {version} (db={expected}, fs={actual})"
            ),
            ExecutorError::StatementExecutionFailed {
                migration_version,
                statement_ordinal,
                message,
                ..
            } => write!(
                f,
                "statement execution failed (version={migration_version}, ordinal={statement_ordinal}): {message}"
            ),
            ExecutorError::TransactionCommitFailed {
                migration_version,
                message,
            } => write!(
                f,
                "transaction commit failed (version={migration_version}): {message}"
            ),
            ExecutorError::RunTrackingFailed {
                run_id,
                operation,
                message,
                original_error,
            } => {
                if let Some(original_error) = original_error {
                    write!(
                        f,
                        "run tracking failed (run_id={run_id}, op={operation}): {message}; original_error={original_error}"
                    )
                } else {
                    write!(
                        f,
                        "run tracking failed (run_id={run_id}, op={operation}): {message}"
                    )
                }
            }
            ExecutorError::TransactionBeginFailed {
                migration_version,
                message,
            } => write!(
                f,
                "transaction begin failed (version={migration_version}): {message}"
            ),
            ExecutorError::TransactionRollbackFailed {
                migration_version,
                message,
                original_error,
            } => write!(
                f,
                "transaction rollback failed (version={migration_version}): {message}; original_error={original_error}"
            ),
        }
    }
}

impl std::error::Error for ExecutorError {}
