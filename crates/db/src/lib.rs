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
