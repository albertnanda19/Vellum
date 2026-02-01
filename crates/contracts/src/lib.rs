pub mod error {
    #[derive(Clone, Debug)]
    pub struct Error {
        message: String,
    }

    impl Error {
        pub fn message(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for Error {}
}

pub use error::Error;

pub mod migration {
    #[derive(Clone, Debug)]
    pub struct MigrationPlan;

    #[derive(Clone, Debug)]
    pub struct MigrationReport;

    pub trait MigrationOrchestrator {
        type Error;

        fn run(&self, plan: MigrationPlan) -> Result<MigrationReport, Self::Error>;
    }
}

pub mod migrations {
    use core::future::Future;
    use core::pin::Pin;

    pub trait DatabaseMigrator {
        type Error;

        fn apply_baseline<'a>(
            &'a self,
        ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'a>>;
    }
}

pub mod sql {
    #[derive(Clone, Debug)]
    pub struct SqlDocument;

    #[derive(Clone, Debug)]
    pub struct SqlAst;

    #[derive(Clone, Debug)]
    pub struct SqlAnalysis;

    pub trait SqlEngine {
        type Error;

        fn parse_and_analyze(&self, input: &str) -> Result<SqlAnalysis, Self::Error>;
    }
}

pub mod schema {
    #[derive(Clone, Debug)]
    pub struct SchemaSnapshot;

    pub trait SchemaIntrospector {
        type Error;

        fn snapshot(&self) -> Result<SchemaSnapshot, Self::Error>;
    }
}

pub mod db {
    pub trait DbConnection {
        type Error;
        type Transaction<'a>: DbTransaction<Error = Self::Error>
        where
            Self: 'a;

        fn begin<'a>(&'a mut self) -> Result<Self::Transaction<'a>, Self::Error>;
    }

    pub trait DbTransaction {
        type Error;

        fn commit(self) -> Result<(), Self::Error>;
        fn rollback(self) -> Result<(), Self::Error>;
    }
}
