use crate::error::{sql_snippet, SqlStatementParseError};
use crate::model::SqlStatement;
use crate::parser::StatementParser;

pub struct PgQueryStatementParser;

impl PgQueryStatementParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PgQueryStatementParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementParser for PgQueryStatementParser {
    fn parse_statements(
        &self,
        sql: &str,
        source_name: Option<&str>,
    ) -> Result<Vec<SqlStatement>, SqlStatementParseError> {
        let parsed = pg_query::parse(sql).map_err(|e| SqlStatementParseError::SqlParseFailed {
            source_name: source_name.map(|s| s.to_string()),
            message: e.to_string(),
            sql_snippet: sql_snippet(sql),
        })?;

        let stmts = &parsed.protobuf.stmts;
        if stmts.is_empty() {
            return Err(SqlStatementParseError::EmptyStatement {
                source_name: source_name.map(|s| s.to_string()),
                statement_index: 0,
            });
        }

        let mut out = Vec::with_capacity(stmts.len());
        let sql_len = sql.len();

        for (idx, raw) in stmts.iter().enumerate() {
            let stmt_location = raw.stmt_location;
            let stmt_len = raw.stmt_len;

            let start: usize = match usize::try_from(stmt_location) {
                Ok(v) => v,
                Err(_) => {
                    return Err(SqlStatementParseError::InvalidStatementLocation {
                        source_name: source_name.map(|s| s.to_string()),
                        statement_index: idx,
                        stmt_location,
                        stmt_len,
                        sql_len,
                    });
                }
            };

            if start > sql_len {
                return Err(SqlStatementParseError::InvalidStatementLocation {
                    source_name: source_name.map(|s| s.to_string()),
                    statement_index: idx,
                    stmt_location,
                    stmt_len,
                    sql_len,
                });
            }

            let extracted = if stmt_len > 0 {
                let len: usize = match usize::try_from(stmt_len) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(SqlStatementParseError::InvalidStatementLocation {
                            source_name: source_name.map(|s| s.to_string()),
                            statement_index: idx,
                            stmt_location,
                            stmt_len,
                            sql_len,
                        });
                    }
                };

                let end = start.saturating_add(len);
                if end > sql_len {
                    return Err(SqlStatementParseError::InvalidStatementLocation {
                        source_name: source_name.map(|s| s.to_string()),
                        statement_index: idx,
                        stmt_location,
                        stmt_len,
                        sql_len,
                    });
                }

                match sql.get(start..end) {
                    Some(s) => s,
                    None => {
                        return Err(SqlStatementParseError::StatementExtractionFailed {
                            source_name: source_name.map(|s| s.to_string()),
                            statement_index: idx,
                            message: format!(
                                "invalid utf-8 slice boundaries (start={start}, end={end})"
                            ),
                        });
                    }
                }
            } else {
                let next_start = stmts
                    .get(idx + 1)
                    .map(|n| n.stmt_location)
                    .unwrap_or(i32::try_from(sql_len).unwrap_or(i32::MAX));

                let next_start_usize = usize::try_from(next_start).unwrap_or(sql_len);
                let end = next_start_usize.min(sql_len);
                if end < start {
                    return Err(SqlStatementParseError::InvalidStatementLocation {
                        source_name: source_name.map(|s| s.to_string()),
                        statement_index: idx,
                        stmt_location,
                        stmt_len,
                        sql_len,
                    });
                }

                match sql.get(start..end) {
                    Some(s) => s,
                    None => {
                        return Err(SqlStatementParseError::StatementExtractionFailed {
                            source_name: source_name.map(|s| s.to_string()),
                            statement_index: idx,
                            message: format!(
                                "invalid utf-8 slice boundaries (start={start}, end={end})"
                            ),
                        });
                    }
                }
            };

            if extracted.is_empty() {
                return Err(SqlStatementParseError::EmptyStatement {
                    source_name: source_name.map(|s| s.to_string()),
                    statement_index: idx,
                });
            }

            let ordinal_usize = idx.saturating_add(1);
            let ordinal = if ordinal_usize > i32::MAX as usize {
                i32::MAX
            } else {
                ordinal_usize as i32
            };

            out.push(SqlStatement {
                ordinal,
                sql: extracted.to_string(),
            });
        }

        Ok(out)
    }
}
