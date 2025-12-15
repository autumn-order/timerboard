use sea_orm_migration::{prelude::*, schema::*};

use super::{
    m20251212_000008_create_ping_format_fields_table::PingFormatField,
    m20251212_000013_create_fleet_table::Fleet,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FleetFieldValue::Table)
                    .if_not_exists()
                    .col(integer(FleetFieldValue::FleetId))
                    .col(integer(FleetFieldValue::FieldId))
                    .col(string(FleetFieldValue::Value))
                    .primary_key(
                        Index::create()
                            .col(FleetFieldValue::FleetId)
                            .col(FleetFieldValue::FieldId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_field_value_fleet_id")
                            .from(FleetFieldValue::Table, FleetFieldValue::FleetId)
                            .to(Fleet::Table, Fleet::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fleet_field_value_field_id")
                            .from(FleetFieldValue::Table, FleetFieldValue::FieldId)
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
            .drop_table(Table::drop().table(FleetFieldValue::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum FleetFieldValue {
    Table,
    FleetId,
    FieldId,
    Value,
}
