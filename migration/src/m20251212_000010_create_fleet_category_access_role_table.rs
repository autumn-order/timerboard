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
                    .table(FleetCategoryAccessRole::Table)
                    .if_not_exists()
                    .col(pk_auto(FleetCategoryAccessRole::Id))
                    .col(integer(FleetCategoryAccessRole::FleetCategoryId))
                    .col(string(FleetCategoryAccessRole::RoleId))
                    .col(boolean(FleetCategoryAccessRole::CanView).default(true))
                    .col(boolean(FleetCategoryAccessRole::CanCreate).default(false))
                    .col(boolean(FleetCategoryAccessRole::CanManage).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_access_role_category_id")
                            .from(
                                FleetCategoryAccessRole::Table,
                                FleetCategoryAccessRole::FleetCategoryId,
                            )
                            .to(FleetCategory::Table, FleetCategory::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_access_role_role_id")
                            .from(
                                FleetCategoryAccessRole::Table,
                                FleetCategoryAccessRole::RoleId,
                            )
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
            .drop_table(
                Table::drop()
                    .table(FleetCategoryAccessRole::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum FleetCategoryAccessRole {
    Table,
    Id,
    FleetCategoryId,
    RoleId,
    CanView,
    CanCreate,
    CanManage,
}
