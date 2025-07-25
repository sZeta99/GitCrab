use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "repositories",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::StringNull),
            ("description", ColType::StringNull),
            ("owner_id", ColType::IntegerNull),
            ("is_private", ColType::BooleanNull),
            ("default_branch", ColType::StringNull),
            ("clone_url", ColType::StringNull),
            ("ssh_url", ColType::StringNull),
            ("size_kb", ColType::IntegerNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "repositories").await
    }
}
