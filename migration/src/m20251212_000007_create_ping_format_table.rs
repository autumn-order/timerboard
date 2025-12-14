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
                    .table(PingFormat::Table)
                    .if_not_exists()
                    .col(pk_auto(PingFormat::Id))
                    .col(string(PingFormat::GuildId))
                    .col(string(PingFormat::Name))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ping_format_guild_id")
                            .from(PingFormat::Table, PingFormat::GuildId)
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
            .drop_table(Table::drop().table(PingFormat::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PingFormat {
    Table,
    Id,
    GuildId,
    Name,
}
