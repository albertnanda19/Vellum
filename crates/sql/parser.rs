use crate::error::SqlStatementParseError;
use crate::model::SqlStatement;

pub trait StatementParser {
    fn parse_statements(
        &self,
        sql: &str,
        source_name: Option<&str>,
    ) -> Result<Vec<SqlStatement>, SqlStatementParseError>;
}
