use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]

pub struct Migration;

#[async_trait::async_trait]

impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiscordUser::Table)
                    .if_not_exists()
                    .col(pk_auto(DiscordUser::Id))
                    .col(big_integer_uniq(DiscordUser::DiscordId))
                    .col(string(DiscordUser::Name))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DiscordUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]

pub enum DiscordUser {
    Table,
    Id,
    DiscordId,
    Name,
}
