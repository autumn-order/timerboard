pub use sea_orm_migration::prelude::*;

mod m20251210_000001_discord_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20251210_000001_discord_user::Migration)]
    }
}
