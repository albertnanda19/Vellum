use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlStatementParseError {
    SqlParseFailed {
        source_name: Option<String>,
        message: String,
        sql_snippet: String,
    },
    InvalidStatementLocation {
        source_name: Option<String>,
        statement_index: usize,
        stmt_location: i32,
        stmt_len: i32,
        sql_len: usize,
    },
    EmptyStatement {
        source_name: Option<String>,
        statement_index: usize,
    },
    StatementExtractionFailed {
        source_name: Option<String>,
        statement_index: usize,
        message: String,
    },
}

impl fmt::Display for SqlStatementParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlStatementParseError::SqlParseFailed {
                source_name,
                message,
                sql_snippet,
            } => {
                if let Some(name) = source_name {
                    write!(f, "sql parse failed ({name}): {message}; sql={sql_snippet}")
                } else {
                    write!(f, "sql parse failed: {message}; sql={sql_snippet}")
                }
            }
            SqlStatementParseError::InvalidStatementLocation {
                source_name,
                statement_index,
                stmt_location,
                stmt_len,
                sql_len,
            } => {
                if let Some(name) = source_name {
                    write!(
                        f,
                        "invalid statement location ({name}) at index {statement_index} (location={stmt_location}, len={stmt_len}, sql_len={sql_len})"
                    )
                } else {
                    write!(
                        f,
                        "invalid statement location at index {statement_index} (location={stmt_location}, len={stmt_len}, sql_len={sql_len})"
                    )
                }
            }
            SqlStatementParseError::EmptyStatement {
                source_name,
                statement_index,
            } => {
                if let Some(name) = source_name {
                    write!(f, "empty statement ({name}) at index {statement_index}")
                } else {
                    write!(f, "empty statement at index {statement_index}")
                }
            }
            SqlStatementParseError::StatementExtractionFailed {
                source_name,
                statement_index,
                message,
            } => {
                if let Some(name) = source_name {
                    write!(
                        f,
                        "statement extraction failed ({name}) at index {statement_index}: {message}"
                    )
                } else {
                    write!(f, "statement extraction failed at index {statement_index}: {message}")
                }
            }
        }
    }
}

impl std::error::Error for SqlStatementParseError {}

pub fn sql_snippet(sql: &str) -> String {
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
