use crate::error::ExecutorError;
use core::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatement {
    pub ordinal: i32,
    pub sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementOutcome {
    pub statement: SqlStatement,
    pub execution_time_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
}

pub fn split_statements(sql: &str) -> Vec<SqlStatement> {
    let mut out = Vec::new();

    for chunk in sql.split(';') {
        let stmt = chunk.trim();
        if stmt.is_empty() {
            continue;
        }

        let next = out.len().saturating_add(1);
        let ordinal = if next > i32::MAX as usize {
            i32::MAX
        } else {
            next as i32
        };
        out.push(SqlStatement {
            ordinal,
            sql: stmt.to_string(),
        });
    }

    out
}

pub fn statement_kind(sql: &str) -> String {
    let token = sql
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_ascii_alphabetic());

    if token.is_empty() {
        return "UNKNOWN".to_string();
    }

    token.to_ascii_uppercase()
}

fn is_forbidden_transaction_control(kind: &str) -> bool {
    matches!(kind, "BEGIN" | "COMMIT" | "ROLLBACK" | "START")
}

pub fn duration_ms(d: Duration) -> i32 {
    let ms = d.as_millis();
    if ms > i32::MAX as u128 {
        i32::MAX
    } else {
        ms as i32
    }
}

pub async fn execute_statement(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    migration_version: i64,
    stmt: &SqlStatement,
) -> Result<i32, ExecutorError> {
    let kind = statement_kind(&stmt.sql);
    if is_forbidden_transaction_control(&kind) {
        return Err(ExecutorError::StatementExecutionFailed {
            migration_version,
            statement_ordinal: stmt.ordinal,
            execution_time_ms: 0,
            statement: stmt.sql.clone(),
            message: "transaction control statements are not allowed inside migration files"
                .to_string(),
        });
    }

    let started = std::time::Instant::now();
    let result = sqlx::query(&stmt.sql).execute(&mut **tx).await;
    let elapsed = duration_ms(started.elapsed());

    match result {
        Ok(_) => Ok(elapsed),
        Err(e) => Err(ExecutorError::StatementExecutionFailed {
            migration_version,
            statement_ordinal: stmt.ordinal,
            execution_time_ms: elapsed,
            statement: stmt.sql.clone(),
            message: e.to_string(),
        }),
    }
}
