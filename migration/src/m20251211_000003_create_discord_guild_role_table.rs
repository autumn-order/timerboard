use sea_orm_migration::{prelude::*, schema::*};

use super::m20251211_000002_create_discord_guild_table::DiscordGuild;

#[derive(DeriveMigrationName)]

pub struct Migration;

#[async_trait::async_trait]

impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiscordGuildRole::Table)
                    .if_not_exists()
                    .col(pk_auto(DiscordGuildRole::Id))
                    .col(string(DiscordGuildRole::GuildId))
                    .col(string_uniq(DiscordGuildRole::RoleId))
                    .col(string(DiscordGuildRole::Name))
                    .col(string(DiscordGuildRole::Color))
                    .col(small_unsigned(DiscordGuildRole::Position))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_discord_guild_role_guild_id")
                            .from(DiscordGuildRole::Table, DiscordGuildRole::GuildId)
                            .to(DiscordGuild::Table, DiscordGuild::GuildId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DiscordGuildRole::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]

pub enum DiscordGuildRole {
    Table,
    Id,
    GuildId,
    RoleId,
    Name,
    Color,
    Position,
}
