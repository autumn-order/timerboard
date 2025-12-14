use sea_orm_migration::{prelude::*, schema::*};

use super::m20251210_000001_create_user_table::User;
use super::m20251211_000003_create_discord_guild_role_table::DiscordGuildRole;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserDiscordGuildRole::Table)
                    .if_not_exists()
                    .col(pk_auto(UserDiscordGuildRole::Id))
                    .col(integer(UserDiscordGuildRole::UserId))
                    .col(string(UserDiscordGuildRole::RoleId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_discord_guild_role_user_id")
                            .from(UserDiscordGuildRole::Table, UserDiscordGuildRole::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_discord_guild_role_role_id")
                            .from(UserDiscordGuildRole::Table, UserDiscordGuildRole::RoleId)
                            .to(DiscordGuildRole::Table, DiscordGuildRole::RoleId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_user_guild_role_unique")
                            .col(UserDiscordGuildRole::UserId)
                            .col(UserDiscordGuildRole::RoleId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserDiscordGuildRole::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum UserDiscordGuildRole {
    Table,
    Id,
    UserId,
    RoleId,
}
