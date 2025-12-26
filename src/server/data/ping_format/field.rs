//! Ping format field data repository for database operations.
//!
//! This module provides the `PingFormatFieldRepository` for managing ping format field
//! records in the database. Fields define the structure and customizable content of ping
//! messages for fleet operations. The repository handles creation, updates, queries, and
//! deletion with proper conversion between entity models and parameter models at the
//! infrastructure boundary.

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};

use crate::{
    model::ping_format::PingFormatFieldType,
    server::{
        error::AppError,
        model::ping_format::{CreateFieldData, PingFormatField, UpdateFieldData},
    },
};

/// Repository providing database operations for ping format field management.
///
/// This struct holds a reference to the database connection and provides methods
/// for creating, reading, updating, and deleting ping format field records. Fields
/// define the structure and content of ping messages for fleet operations.
pub struct PingFormatFieldRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PingFormatFieldRepository<'a> {
    /// Creates a new PingFormatFieldRepository instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `PingFormatFieldRepository` - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new ping format field.
    ///
    /// Inserts a new field into the database for the specified ping format. Fields
    /// are ordered by priority when displayed or processed. For text type fields,
    /// also creates default field value records.
    ///
    /// # Arguments
    /// - `guild_id` - The guild ID for scoping verification
    /// - `ping_format_id` - ID of the ping format this field belongs to
    /// - `data` - Field data containing name, priority, field_type, and default_field_values
    ///
    /// # Returns
    /// - `Ok(PingFormatField)` - The created ping format field with generated ID and default values
    /// - `Err(AppError)` - Database error during insert operation or ping format not in guild
    pub async fn create(
        &self,
        guild_id: u64,
        ping_format_id: i32,
        data: CreateFieldData,
    ) -> Result<PingFormatField, AppError> {
        // Verify the ping format belongs to the guild
        let ping_format = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::Id.eq(ping_format_id))
            .filter(entity::ping_format::Column::GuildId.eq(guild_id.to_string()))
            .one(self.db)
            .await?;

        if ping_format.is_none() {
            return Err(AppError::NotFound(format!(
                "Ping format ID {} not found for guild ID {}",
                ping_format_id, guild_id
            )));
        }

        let field_type_str = match data.field_type {
            PingFormatFieldType::Text => "text",
            PingFormatFieldType::Bool => "bool",
        };

        let entity = entity::ping_format_field::ActiveModel {
            ping_format_id: ActiveValue::Set(ping_format_id),
            name: ActiveValue::Set(data.name),
            priority: ActiveValue::Set(data.priority),
            field_type: ActiveValue::Set(field_type_str.to_string()),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        // Create default field values if field type is text and values are provided
        if matches!(data.field_type, PingFormatFieldType::Text) {
            for value in &data.default_field_values {
                entity::ping_format_field_value::ActiveModel {
                    ping_format_field_id: ActiveValue::Set(entity.id.to_string()),
                    value: ActiveValue::Set(value.clone()),
                    ..Default::default()
                }
                .insert(self.db)
                .await?;
            }
        }

        Ok(PingFormatField::from_entity(
            entity,
            data.default_field_values,
        )?)
    }

    /// Gets all fields for a ping format, ordered by priority.
    ///
    /// Returns all fields belonging to the specified ping format, sorted by priority
    /// in ascending order (lowest priority first). Used for displaying field lists and
    /// processing ping messages in the correct order. For text type fields, also fetches
    /// their default values.
    ///
    /// # Arguments
    /// - `guild_id` - The guild ID for scoping verification
    /// - `ping_format_id` - ID of the ping format to get fields for
    ///
    /// # Returns
    /// - `Ok(Vec<PingFormatField>)` - Vector of fields ordered by priority with default values
    /// - `Err(AppError)` - Database error during query or ping format not in guild
    pub async fn get_by_ping_format_id(
        &self,
        guild_id: u64,
        ping_format_id: i32,
    ) -> Result<Vec<PingFormatField>, AppError> {
        // Verify the ping format belongs to the guild
        let ping_format = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::Id.eq(ping_format_id))
            .filter(entity::ping_format::Column::GuildId.eq(guild_id.to_string()))
            .one(self.db)
            .await?;

        if ping_format.is_none() {
            return Err(AppError::NotFound(format!(
                "Ping format ID {} not found for guild ID {}",
                ping_format_id, guild_id
            )));
        }

        let entities = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format_id))
            .order_by_asc(entity::ping_format_field::Column::Priority)
            .all(self.db)
            .await?;

        let mut result = Vec::new();
        for entity in entities {
            let default_field_values = if entity.field_type == "text" {
                // Fetch default values for text fields
                let value_entities = entity::prelude::PingFormatFieldValue::find()
                    .filter(
                        entity::ping_format_field_value::Column::PingFormatFieldId
                            .eq(entity.id.to_string()),
                    )
                    .all(self.db)
                    .await?;

                value_entities.into_iter().map(|v| v.value).collect()
            } else {
                // Bool fields don't have default values
                Vec::new()
            };

            result.push(PingFormatField::from_entity(entity, default_field_values)?);
        }

        Ok(result)
    }

    /// Updates a ping format field's name, priority, field_type, and default values.
    ///
    /// Updates all editable properties of an existing field. For text type fields,
    /// deletes all existing default values and re-inserts the new ones.
    ///
    /// # Arguments
    /// - `guild_id` - The guild ID for scoping verification
    /// - `id` - ID of the field to update
    /// - `data` - Field data containing name, priority, field_type, and default_field_values
    ///
    /// # Returns
    /// - `Ok(PingFormatField)` - The updated field with new default values
    /// - `Err(AppError::NotFound)` - No field exists with the specified ID in the guild
    /// - `Err(AppError)` - Other database error during update operation
    pub async fn update(
        &self,
        guild_id: u64,
        id: i32,
        data: UpdateFieldData,
    ) -> Result<PingFormatField, AppError> {
        // Find the field and verify it belongs to the guild through its ping format
        let field_with_format = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::Id.eq(id))
            .find_also_related(entity::prelude::PingFormat)
            .one(self.db)
            .await?;

        let (field, ping_format) = field_with_format.ok_or_else(|| {
            AppError::NotFound(format!("Ping format field with id {} not found", id))
        })?;

        let ping_format = ping_format
            .ok_or_else(|| AppError::NotFound(format!("Ping format for field {} not found", id)))?;

        if ping_format.guild_id != guild_id.to_string() {
            return Err(AppError::NotFound(format!(
                "Ping format field ID {} not found for guild ID {}",
                id, guild_id
            )));
        }

        let field_type_str = match data.field_type {
            PingFormatFieldType::Text => "text",
            PingFormatFieldType::Bool => "bool",
        };

        let mut active_model: entity::ping_format_field::ActiveModel = field.into();
        active_model.name = ActiveValue::Set(data.name);
        active_model.priority = ActiveValue::Set(data.priority);
        active_model.field_type = ActiveValue::Set(field_type_str.to_string());

        let entity = active_model.update(self.db).await?;

        // Delete all existing default values
        entity::prelude::PingFormatFieldValue::delete_many()
            .filter(
                entity::ping_format_field_value::Column::PingFormatFieldId
                    .eq(entity.id.to_string()),
            )
            .exec(self.db)
            .await?;

        // Re-insert default values if field type is text
        if matches!(data.field_type, PingFormatFieldType::Text) {
            for value in &data.default_field_values {
                entity::ping_format_field_value::ActiveModel {
                    ping_format_field_id: ActiveValue::Set(entity.id.to_string()),
                    value: ActiveValue::Set(value.clone()),
                    ..Default::default()
                }
                .insert(self.db)
                .await?;
            }
        }

        Ok(PingFormatField::from_entity(
            entity,
            data.default_field_values,
        )?)
    }

    /// Deletes a ping format field.
    ///
    /// Deletes the field with the specified ID. Associated default field values
    /// and fleet field values are automatically deleted due to CASCADE foreign key constraint.
    ///
    /// # Arguments
    /// - `guild_id` - The guild ID for scoping verification
    /// - `id` - ID of the field to delete
    ///
    /// # Returns
    /// - `Ok(())` - Field deleted successfully
    /// - `Err(AppError::NotFound)` - Field not found or doesn't belong to the guild
    /// - `Err(AppError)` - Database error during delete operation
    pub async fn delete(&self, guild_id: u64, id: i32) -> Result<(), AppError> {
        // Verify the field belongs to a ping format in the guild
        let field_with_format = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::Id.eq(id))
            .find_also_related(entity::prelude::PingFormat)
            .one(self.db)
            .await?;

        let (field, ping_format) = field_with_format.ok_or_else(|| {
            AppError::NotFound(format!("Ping format field with id {} not found", id))
        })?;

        let ping_format = ping_format
            .ok_or_else(|| AppError::NotFound(format!("Ping format for field {} not found", id)))?;

        if ping_format.guild_id != guild_id.to_string() {
            return Err(AppError::NotFound(format!(
                "Ping format field ID {} not found for guild ID {}",
                id, guild_id
            )));
        }

        entity::prelude::PingFormatField::delete_by_id(field.id)
            .exec(self.db)
            .await?;

        Ok(())
    }
}
