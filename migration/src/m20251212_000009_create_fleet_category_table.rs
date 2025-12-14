use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m20251211_000002_create_discord_guild_table::DiscordGuild,
    m20251212_000007_create_ping_format_table::PingFormat,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FleetCategory::Table)
                    .if_not_exists()
                    .col(pk_auto(FleetCategory::Id))
                    .col(string(FleetCategory::GuildId))
                    .col(integer(FleetCategory::PingFormatId))
                    .col(string(FleetCategory::Name))
                    .col(integer_null(FleetCategory::PingCooldown))
                    .col(integer_null(FleetCategory::PingReminder))
                    .col(integer_null(FleetCategory::MaxPrePing))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_guild_id")
                            .from(FleetCategory::Table, FleetCategory::GuildId)
                            .to(DiscordGuild::Table, DiscordGuild::GuildId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_category_ping_format_id")
                            .from(FleetCategory::Table, FleetCategory::PingFormatId)
                            .to(PingFormat::Table, PingFormat::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FleetCategory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum FleetCategory {
    Table,
    Id,
    GuildId,
    PingFormatId,
    Name,
    PingCooldown,
    PingReminder,
    MaxPrePing,
}
