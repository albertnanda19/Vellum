use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockError {
    MigrationLockUnavailable {
        timeout_ms: u64,
    },
    LockAcquireFailed {
        message: String,
    },
    LockReleaseFailed {
        message: String,
    },
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockError::MigrationLockUnavailable { timeout_ms } => {
                write!(f, "migration lock unavailable (timeout_ms={timeout_ms})")
            }
            LockError::LockAcquireFailed { message } => {
                write!(f, "lock acquire failed: {message}")
            }
            LockError::LockReleaseFailed { message } => {
                write!(f, "lock release failed: {message}")
            }
        }
    }
}

impl std::error::Error for LockError {}
