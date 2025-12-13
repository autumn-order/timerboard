use sea_orm_migration::{prelude::*, schema::*};

use super::m20251212_000008_create_ping_format_table::PingFormat;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PingFormatField::Table)
                    .if_not_exists()
                    .col(pk_auto(PingFormatField::Id))
                    .col(big_unsigned(PingFormatField::PingFormatId))
                    .col(string(PingFormatField::Name))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ping_format_field_ping_format_id")
                            .from(PingFormatField::Table, PingFormatField::PingFormatId)
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
            .drop_table(Table::drop().table(PingFormatField::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PingFormatField {
    Table,
    Id,
    PingFormatId,
    Name,
}
