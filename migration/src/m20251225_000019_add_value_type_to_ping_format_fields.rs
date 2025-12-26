use sea_orm_migration::{prelude::*, schema::*};

use crate::m20251212_000008_create_ping_format_fields_table::PingFormatField;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PingFormatField::Table)
                    .drop_column(PingFormatField::DefaultValue)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PingFormatField::Table)
                    .add_column(string(PingFormatField::FieldType).default("text"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PingFormatField::Table)
                    .drop_column(PingFormatField::FieldType)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PingFormatField::Table)
                    .add_column(string_null(PingFormatField::DefaultValue))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
