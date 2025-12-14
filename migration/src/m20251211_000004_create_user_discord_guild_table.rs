use sea_orm_migration::{prelude::*, schema::*};

use super::m20251210_000001_create_user_table::User;
use super::m20251211_000002_create_discord_guild_table::DiscordGuild;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserDiscordGuild::Table)
                    .if_not_exists()
                    .col(pk_auto(UserDiscordGuild::Id))
                    .col(integer(UserDiscordGuild::UserId))
                    .col(string(UserDiscordGuild::GuildId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_discord_guild_user_id")
                            .from(UserDiscordGuild::Table, UserDiscordGuild::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_discord_guild_guild_id")
                            .from(UserDiscordGuild::Table, UserDiscordGuild::GuildId)
                            .to(DiscordGuild::Table, DiscordGuild::GuildId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_user_guild_unique")
                            .col(UserDiscordGuild::UserId)
                            .col(UserDiscordGuild::GuildId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserDiscordGuild::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum UserDiscordGuild {
    Table,
    Id,
    UserId,
    GuildId,
}
