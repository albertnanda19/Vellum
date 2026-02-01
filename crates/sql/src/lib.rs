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
