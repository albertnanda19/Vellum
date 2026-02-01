pub use vellum_contracts::db;
pub use vellum_contracts::migration;
pub use vellum_contracts::migrations;
pub use vellum_contracts::schema;
pub use vellum_contracts::sql;
pub use vellum_contracts::Error;

pub mod orchestrator {
    use vellum_contracts::db::DbConnection;
    use vellum_contracts::migration::{MigrationOrchestrator, MigrationPlan, MigrationReport};
    use vellum_contracts::schema::SchemaIntrospector;
    use vellum_contracts::sql::SqlEngine;
    use vellum_contracts::Error;

    #[allow(dead_code)]
    pub struct Orchestrator<DB, SCH, SQL>
    where
        DB: DbConnection<Error = Error>,
        SCH: SchemaIntrospector<Error = Error>,
        SQL: SqlEngine<Error = Error>,
    {
        db: DB,
        schema: SCH,
        sql: SQL,
    }

    impl<DB, SCH, SQL> Orchestrator<DB, SCH, SQL>
    where
        DB: DbConnection<Error = Error>,
        SCH: SchemaIntrospector<Error = Error>,
        SQL: SqlEngine<Error = Error>,
    {
        pub fn new(db: DB, schema: SCH, sql: SQL) -> Self {
            Self { db, schema, sql }
        }
    }

    impl<DB, SCH, SQL> MigrationOrchestrator for Orchestrator<DB, SCH, SQL>
    where
        DB: DbConnection<Error = Error>,
        SCH: SchemaIntrospector<Error = Error>,
        SQL: SqlEngine<Error = Error>,
    {
        type Error = Error;

        fn run(&self, _plan: MigrationPlan) -> Result<MigrationReport, Self::Error> {
            Ok(MigrationReport)
        }
    }
}

pub use orchestrator::Orchestrator;

pub use vellum_contracts::migration::{MigrationOrchestrator, MigrationPlan, MigrationReport};

pub mod bootstrap {
    use core::future::Future;
    use core::pin::Pin;
    use vellum_contracts::migrations::DatabaseMigrator;
    use vellum_contracts::Error;

    pub fn apply_baseline<'a, M>(
        migrator: &'a M,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
    where
        M: DatabaseMigrator<Error = Error>,
    {
        migrator.apply_baseline()
    }
}

pub mod commands {
    use vellum_contracts::migration::{MigrationOrchestrator, MigrationPlan, MigrationReport};

    pub fn migrate<O>(orchestrator: &O) -> Result<MigrationReport, O::Error>
    where
        O: MigrationOrchestrator,
    {
        orchestrator.run(MigrationPlan)
    }

    pub fn dry_run<O>(orchestrator: &O) -> Result<MigrationReport, O::Error>
    where
        O: MigrationOrchestrator,
    {
        orchestrator.run(MigrationPlan)
    }

    pub fn status<O>(orchestrator: &O) -> Result<MigrationReport, O::Error>
    where
        O: MigrationOrchestrator,
    {
        orchestrator.run(MigrationPlan)
    }
}

#[cfg(feature = "runtime")]
pub mod runtime {
    use crate::orchestrator::Orchestrator;
    use core::future::Future;
    use core::pin::Pin;
    use crate::Error;

    pub fn build_orchestrator(
    ) -> Orchestrator<vellum_db::DefaultDbConnection, vellum_schema::DefaultSchemaIntrospector, vellum_sql::DefaultSqlEngine>
    {
        Orchestrator::new(
            vellum_db::DefaultDbConnection::new(),
            vellum_schema::DefaultSchemaIntrospector::new(),
            vellum_sql::DefaultSqlEngine::new(),
        )
    }

    pub fn build_migrator<'a>(
        database_url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<vellum_db::SqlxDatabaseMigrator, Error>> + Send + 'a>>
    {
        Box::pin(async move { vellum_db::SqlxDatabaseMigrator::connect(database_url).await })
    }

    pub fn run_migrations<'a>(
        database_url: &'a str,
        migrations_dir: &'a std::path::Path,
        vellum_version: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<vellum_executor::RunReport, Error>> + Send + 'a>> {
        Box::pin(async move {
            let migrations = vellum_migration::discover_migrations(migrations_dir)
                .map_err(|e| Error::message(e.to_string()))?;

            let pool = sqlx::PgPool::connect(database_url)
                .await
                .map_err(|e| Error::message(e.to_string()))?;

            let runner = vellum_executor::Runner::new(pool, vellum_version);
            runner.run(&migrations).await.map_err(|e| Error::message(e.to_string()))
        })
    }
}
