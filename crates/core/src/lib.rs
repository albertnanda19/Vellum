pub use vellum_contracts::db;
pub use vellum_contracts::migration;
pub use vellum_contracts::schema;
pub use vellum_contracts::sql;
pub use vellum_contracts::Error;

pub mod orchestrator {
    use vellum_contracts::db::DbConnection;
    use vellum_contracts::migration::{MigrationOrchestrator, MigrationPlan, MigrationReport};
    use vellum_contracts::schema::SchemaIntrospector;
    use vellum_contracts::sql::SqlEngine;
    use vellum_contracts::Error;

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

    pub fn build_orchestrator(
    ) -> Orchestrator<vellum_db::DefaultDbConnection, vellum_schema::DefaultSchemaIntrospector, vellum_sql::DefaultSqlEngine>
    {
        Orchestrator::new(
            vellum_db::DefaultDbConnection::new(),
            vellum_schema::DefaultSchemaIntrospector::new(),
            vellum_sql::DefaultSqlEngine::new(),
        )
    }
}
