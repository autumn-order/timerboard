use sea_orm_migration::{prelude::*, schema::*};

use crate::m20251212_000008_create_ping_format_fields_table::PingFormatField;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PingFormatFieldValue::Table)
                    .if_not_exists()
                    .col(pk_auto(PingFormatFieldValue::Id))
                    .col(string(PingFormatFieldValue::PingFormatFieldId))
                    .col(string(PingFormatFieldValue::Value))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ping_format_field_value_ping_format_field_id")
                            .from(
                                PingFormatFieldValue::Table,
                                PingFormatFieldValue::PingFormatFieldId,
                            )
                            .to(PingFormatField::Table, PingFormatField::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PingFormatFieldValue::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PingFormatFieldValue {
    Table,
    Id,
    PingFormatFieldId,
    Value,
}
