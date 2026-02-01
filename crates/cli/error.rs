use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    UserError = 1,
    MigrationFailed = 2,
    LockUnavailable = 3,
}

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone)]
pub struct CliError {
    code: ExitCode,
    message: String,
    hint: Option<String>,
}

impl CliError {
    pub fn user_error(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code: ExitCode::UserError,
            message: message.into(),
            hint: Some(hint.into()),
        }
    }

    pub fn migration_failed(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code: ExitCode::MigrationFailed,
            message: message.into(),
            hint: Some(hint.into()),
        }
    }

    pub fn lock_unavailable(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code: ExitCode::LockUnavailable,
            message: message.into(),
            hint: Some(hint.into()),
        }
    }

    pub fn exit_code(&self) -> i32 {
        self.code.as_i32()
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(hint) = &self.hint {
            write!(f, "{}\n{}", self.message, hint)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for CliError {}
