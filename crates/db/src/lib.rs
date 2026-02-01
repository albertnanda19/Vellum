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

pub struct SqlxDatabaseMigrator {
    pool: sqlx::PgPool,
}

impl SqlxDatabaseMigrator {
    pub async fn connect(database_url: &str) -> Result<Self, vellum_contracts::Error> {
        let pool = sqlx::PgPool::connect(database_url)
            .await
            .map_err(|e| vellum_contracts::Error::message(e.to_string()))?;

        Ok(Self { pool })
    }
}

impl vellum_contracts::migrations::DatabaseMigrator for SqlxDatabaseMigrator {
    type Error = vellum_contracts::Error;

    fn apply_baseline<'a>(
        &'a self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<(), Self::Error>> + Send + 'a>,
    > {
        Box::pin(async move {
            let sql = include_str!("../migrations/001_init_schema.sql");

            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|e| vellum_contracts::Error::message(e.to_string()))?;

            for chunk in sql.split(';') {
                let stmt = chunk.trim();
                if stmt.is_empty() {
                    continue;
                }

                let stmt_upper = stmt.to_ascii_uppercase();
                if stmt_upper == "BEGIN" || stmt_upper == "COMMIT" {
                    continue;
                }

                if let Err(e) = sqlx::query(stmt).execute(&mut *tx).await {
                    let _ = tx.rollback().await;
                    return Err(vellum_contracts::Error::message(e.to_string()));
                }
            }

            tx.commit()
                .await
                .map_err(|e| vellum_contracts::Error::message(e.to_string()))?;

            Ok(())
        })
    }
}
