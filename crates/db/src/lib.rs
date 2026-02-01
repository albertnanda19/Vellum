pub mod contract {
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

pub use contract::{DbConnection, DbTransaction};

pub struct DefaultDbConnection;

impl DefaultDbConnection {
    pub fn new() -> Self {
        Self
    }
}

pub struct DefaultDbTransaction;

impl vellum_contracts::db::DbTransaction for DefaultDbTransaction {
    type Error = vellum_contracts::Error;

    fn commit(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn rollback(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl vellum_contracts::db::DbConnection for DefaultDbConnection {
    type Error = vellum_contracts::Error;
    type Transaction<'a> = DefaultDbTransaction where Self: 'a;

    fn begin<'a>(&'a mut self) -> Result<Self::Transaction<'a>, Self::Error> {
        Ok(DefaultDbTransaction)
    }
}
