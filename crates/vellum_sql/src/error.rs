use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SqlParseError {
    #[error("sql parse failed: {message}")]
    ParseFailed {
        message: String,
        position: Option<usize>,
    },
}

impl SqlParseError {
    pub fn message(&self) -> &str {
        match self {
            SqlParseError::ParseFailed { message, .. } => message,
        }
    }

    pub fn position(&self) -> Option<usize> {
        match self {
            SqlParseError::ParseFailed { position, .. } => *position,
        }
    }
}
