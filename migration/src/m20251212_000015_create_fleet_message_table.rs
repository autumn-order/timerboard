use sea_orm_migration::{prelude::*, schema::*};

use super::m20251212_000013_create_fleet_table::Fleet;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table
        manager
            .create_table(
                Table::create()
                    .table(FleetMessage::Table)
                    .if_not_exists()
                    .col(pk_auto(FleetMessage::Id))
                    .col(integer(FleetMessage::FleetId))
                    .col(string(FleetMessage::ChannelId))
                    .col(string(FleetMessage::MessageId))
                    .col(string(FleetMessage::MessageType).not_null())
                    .col(
                        timestamp(FleetMessage::CreatedAt)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_message_fleet_id")
                            .from(FleetMessage::Table, FleetMessage::FleetId)
                            .to(Fleet::Table, Fleet::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for fleet_id lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_fleet_message_fleet_id")
                    .table(FleetMessage::Table)
                    .col(FleetMessage::FleetId)
                    .to_owned(),
            )
            .await?;

        // Create unique index for one message type per channel per fleet
        manager
            .create_index(
                Index::create()
                    .name("idx_fleet_message_unique")
                    .table(FleetMessage::Table)
                    .col(FleetMessage::FleetId)
                    .col(FleetMessage::ChannelId)
                    .col(FleetMessage::MessageType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_fleet_message_unique")
                    .table(FleetMessage::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_fleet_message_fleet_id")
                    .table(FleetMessage::Table)
                    .to_owned(),
            )
            .await?;

        // Drop table
        manager
            .drop_table(Table::drop().table(FleetMessage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum FleetMessage {
    Table,
    Id,
    FleetId,
    ChannelId,
    MessageId,
    MessageType,
    CreatedAt,
}
