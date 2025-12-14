use sea_orm_migration::{prelude::*, schema::*};

use super::m20251211_000006_create_discord_guild_channel_table::DiscordGuildChannel;
use super::m20251212_000009_create_fleet_category_table::FleetCategory;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FleetCategoryChannel::Table)
                    .if_not_exists()
                    .col(pk_auto(FleetCategoryChannel::Id))
                    .col(integer(FleetCategoryChannel::FleetCategoryId))
                    .col(string(FleetCategoryChannel::ChannelId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_channel_category_id")
                            .from(
                                FleetCategoryChannel::Table,
                                FleetCategoryChannel::FleetCategoryId,
                            )
                            .to(FleetCategory::Table, FleetCategory::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_channel_channel_id")
                            .from(FleetCategoryChannel::Table, FleetCategoryChannel::ChannelId)
                            .to(DiscordGuildChannel::Table, DiscordGuildChannel::ChannelId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FleetCategoryChannel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum FleetCategoryChannel {
    Table,
    Id,
    FleetCategoryId,
    ChannelId,
}
