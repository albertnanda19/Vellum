use crate::SqlParseError;

#[derive(Debug)]
pub struct ParsedSql {
    sql: String,
    ast: pg_query::ParseResult,
}

impl ParsedSql {
    pub fn sql(&self) -> &str {
        &self.sql
    }

    pub(crate) fn ast(&self) -> &pg_query::ParseResult {
        &self.ast
    }
}

pub fn parse_sql(sql: &str) -> Result<ParsedSql, SqlParseError> {
    let parsed = pg_query::parse(sql).map_err(|e| SqlParseError::ParseFailed {
        message: e.to_string(),
        position: extract_position(&e),
    })?;

    Ok(ParsedSql {
        sql: sql.to_string(),
        ast: parsed,
    })
}

fn extract_position(err: &pg_query::Error) -> Option<usize> {
    let msg = err.to_string();

    // Best-effort parse from common postgres/pg_query format:
    // "... at or near \"X\" at character N"
    let needle = " at character ";
    let idx = msg.rfind(needle)?;
    let start = idx + needle.len();
    let digits = msg[start..].trim();

    let mut end = 0usize;
    for (i, ch) in digits.char_indices() {
        if !ch.is_ascii_digit() {
            break;
        }
        end = i + ch.len_utf8();
    }
    if end == 0 {
        return None;
    }

    digits[..end].parse::<usize>().ok()
}
