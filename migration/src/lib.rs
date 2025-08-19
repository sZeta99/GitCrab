#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20250411_134017_git_repos;

mod m20250819_161131_sshes;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20250411_134017_git_repos::Migration),
            Box::new(m20250819_161131_sshes::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
