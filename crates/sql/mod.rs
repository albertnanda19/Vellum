pub mod contract {
    #[derive(Clone, Debug)]
    pub struct SqlDocument;

    #[derive(Clone, Debug)]
    pub struct SqlAst;

    #[derive(Clone, Debug)]
    pub struct SqlAnalysis;

    pub trait SqlParser {
        type Error;

        fn parse(&self, input: &str) -> Result<SqlAst, Self::Error>;
    }

    pub trait SqlAnalyzer {
        type Error;

        fn analyze(&self, ast: &SqlAst) -> Result<SqlAnalysis, Self::Error>;
    }
}

pub use contract::{SqlAnalysis, SqlAnalyzer, SqlAst, SqlDocument, SqlParser};

pub struct DefaultSqlEngine;

impl DefaultSqlEngine {
    pub fn new() -> Self {
        Self
    }
}

impl vellum_contracts::sql::SqlEngine for DefaultSqlEngine {
    type Error = vellum_contracts::Error;

    fn parse_and_analyze(&self, _input: &str) -> Result<vellum_contracts::sql::SqlAnalysis, Self::Error> {
        Ok(vellum_contracts::sql::SqlAnalysis)
    }
}

pub mod error;
pub mod model;
pub mod parser;
pub mod pg_query;

pub use error::SqlStatementParseError;
pub use model::SqlStatement;
pub use parser::StatementParser;
pub use pg_query::PgQueryStatementParser;
