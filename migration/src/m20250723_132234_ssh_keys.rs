use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "ssh_keys",
            &[
            
            ("id", ColType::PkAuto),
            
            ("user_id", ColType::IntegerNull),
            ("title", ColType::StringNull),
            ("key_type", ColType::StringNull),
            ("public_key", ColType::StringNull),
            ("fingerprint", ColType::StringNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "ssh_keys").await
    }
}
