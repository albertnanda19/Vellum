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

pub use migration::{MigrationOrchestrator, MigrationPlan, MigrationReport};

pub mod commands {
    pub fn migrate() {}

    pub fn dry_run() {}

    pub fn status() {}
}
