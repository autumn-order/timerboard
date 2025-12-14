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
                    .table(DiscordGuildChannel::Table)
                    .if_not_exists()
                    .col(pk_auto(DiscordGuildChannel::Id))
                    .col(string(DiscordGuildChannel::GuildId))
                    .col(string_uniq(DiscordGuildChannel::ChannelId))
                    .col(string(DiscordGuildChannel::Name))
                    .col(integer(DiscordGuildChannel::Position))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_discord_guild_channel_guild_id")
                            .from(DiscordGuildChannel::Table, DiscordGuildChannel::GuildId)
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
            .drop_table(Table::drop().table(DiscordGuildChannel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum DiscordGuildChannel {
    Table,
    Id,
    GuildId,
    ChannelId,
    Name,
    Position,
}
