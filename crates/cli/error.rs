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
    title: String,
    reason: Option<String>,
    meaning: Option<String>,
    action: Option<String>,
}

impl CliError {
    pub fn user_error(title: impl Into<String>) -> Self {
        Self {
            code: ExitCode::UserError,
            title: title.into(),
            reason: None,
            meaning: None,
            action: None,
        }
    }

    pub fn migration_failed(title: impl Into<String>) -> Self {
        Self {
            code: ExitCode::MigrationFailed,
            title: title.into(),
            reason: None,
            meaning: None,
            action: None,
        }
    }

    pub fn lock_unavailable(title: impl Into<String>) -> Self {
        Self {
            code: ExitCode::LockUnavailable,
            title: title.into(),
            reason: None,
            meaning: None,
            action: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn with_meaning(mut self, meaning: impl Into<String>) -> Self {
        self.meaning = Some(meaning.into());
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn meaning(&self) -> Option<&str> {
        self.meaning.as_deref()
    }

    pub fn action(&self) -> Option<&str> {
        self.action.as_deref()
    }

    pub fn exit_code(&self) -> i32 {
        self.code.as_i32()
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl std::error::Error for CliError {}
