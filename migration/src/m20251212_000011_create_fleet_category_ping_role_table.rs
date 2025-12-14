use sea_orm_migration::{prelude::*, schema::*};

use super::m20251211_000003_create_discord_guild_role_table::DiscordGuildRole;
use super::m20251212_000009_create_fleet_category_table::FleetCategory;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FleetCategoryPingRole::Table)
                    .if_not_exists()
                    .col(pk_auto(FleetCategoryPingRole::Id))
                    .col(integer(FleetCategoryPingRole::FleetCategoryId))
                    .col(string(FleetCategoryPingRole::RoleId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_ping_role_category_id")
                            .from(
                                FleetCategoryPingRole::Table,
                                FleetCategoryPingRole::FleetCategoryId,
                            )
                            .to(FleetCategory::Table, FleetCategory::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_ping_role_role_id")
                            .from(FleetCategoryPingRole::Table, FleetCategoryPingRole::RoleId)
                            .to(DiscordGuildRole::Table, DiscordGuildRole::RoleId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FleetCategoryPingRole::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum FleetCategoryPingRole {
    Table,
    Id,
    FleetCategoryId,
    RoleId,
}
