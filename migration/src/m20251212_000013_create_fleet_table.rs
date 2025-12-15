use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m20251210_000001_create_user_table::User,
    m20251212_000009_create_fleet_category_table::FleetCategory,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fleet::Table)
                    .if_not_exists()
                    .col(pk_auto(Fleet::Id))
                    .col(integer(Fleet::CategoryId))
                    .col(string(Fleet::Name))
                    .col(string(Fleet::CommanderId))
                    .col(timestamp(Fleet::FleetTime))
                    .col(text_null(Fleet::Description))
                    .col(
                        timestamp(Fleet::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_id")
                            .from(Fleet::Table, Fleet::CategoryId)
                            .to(FleetCategory::Table, FleetCategory::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_commander_id")
                            .from(Fleet::Table, Fleet::CommanderId)
                            .to(User::Table, User::DiscordId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Fleet::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Fleet {
    Table,
    Id,
    CategoryId,
    Name,
    CommanderId,
    FleetTime,
    Description,
    CreatedAt,
}
