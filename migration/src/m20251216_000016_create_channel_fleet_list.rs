use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChannelFleetList::Table)
                    .if_not_exists()
                    .col(pk_auto(ChannelFleetList::Id))
                    .col(string(ChannelFleetList::ChannelId))
                    .col(string(ChannelFleetList::MessageId))
                    .col(timestamp(ChannelFleetList::LastMessageAt))
                    .col(timestamp(ChannelFleetList::CreatedAt))
                    .col(timestamp(ChannelFleetList::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Create unique index on channel_id
        manager
            .create_index(
                Index::create()
                    .name("idx_channel_fleet_list_channel_id")
                    .table(ChannelFleetList::Table)
                    .col(ChannelFleetList::ChannelId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_channel_fleet_list_channel_id")
                    .table(ChannelFleetList::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ChannelFleetList::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChannelFleetList {
    Table,
    Id,
    ChannelId,
    MessageId,
    LastMessageAt,
    CreatedAt,
    UpdatedAt,
}
