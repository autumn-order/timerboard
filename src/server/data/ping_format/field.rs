//! Ping format field data repository for database operations.
//!
//! This module provides the `PingFormatFieldRepository` for managing ping format field
//! records in the database. Fields define the structure and customizable content of ping
//! messages for fleet operations. The repository handles creation, updates, queries, and
//! deletion with proper conversion between entity models and parameter models at the
//! infrastructure boundary.

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

use crate::server::model::ping_format::{
    CreatePingFormatFieldParam, PingFormatField, UpdatePingFormatFieldParam,
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
    /// are ordered by priority when displayed or processed.
    ///
    /// # Arguments
    /// - `param` - Create parameters containing ping_format_id, name, priority, and default_value
    ///
    /// # Returns
    /// - `Ok(PingFormatField)` - The created ping format field with generated ID
    /// - `Err(DbErr)` - Database error during insert operation
    pub async fn create(
        &self,
        param: CreatePingFormatFieldParam,
    ) -> Result<PingFormatField, DbErr> {
        let entity = entity::ping_format_field::ActiveModel {
            ping_format_id: ActiveValue::Set(param.ping_format_id),
            name: ActiveValue::Set(param.name),
            priority: ActiveValue::Set(param.priority),
            default_value: ActiveValue::Set(param.default_value),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        Ok(PingFormatField::from_entity(entity))
    }

    /// Gets all fields for a ping format, ordered by priority.
    ///
    /// Returns all fields belonging to the specified ping format, sorted by priority
    /// in ascending order (lowest priority first). Used for displaying field lists and
    /// processing ping messages in the correct order.
    ///
    /// # Arguments
    /// - `ping_format_id` - ID of the ping format to get fields for
    ///
    /// # Returns
    /// - `Ok(Vec<PingFormatField>)` - Vector of fields ordered by priority
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<PingFormatField>, DbErr> {
        let entities = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format_id))
            .order_by_asc(entity::ping_format_field::Column::Priority)
            .all(self.db)
            .await?;

        Ok(entities
            .into_iter()
            .map(PingFormatField::from_entity)
            .collect())
    }

    /// Updates a ping format field's name, priority, and default value.
    ///
    /// Updates all editable properties of an existing field. All properties must be
    /// provided even if only some are changing.
    ///
    /// # Arguments
    /// - `param` - Update parameters containing id, name, priority, and default_value
    ///
    /// # Returns
    /// - `Ok(PingFormatField)` - The updated field
    /// - `Err(DbErr::RecordNotFound)` - No field exists with the specified ID
    /// - `Err(DbErr)` - Other database error during update operation
    pub async fn update(
        &self,
        param: UpdatePingFormatFieldParam,
    ) -> Result<PingFormatField, DbErr> {
        let field = entity::prelude::PingFormatField::find_by_id(param.id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Ping format field with id {} not found",
                param.id
            )))?;

        let mut active_model: entity::ping_format_field::ActiveModel = field.into();
        active_model.name = ActiveValue::Set(param.name);
        active_model.priority = ActiveValue::Set(param.priority);
        active_model.default_value = ActiveValue::Set(param.default_value);

        let entity = active_model.update(self.db).await?;

        Ok(PingFormatField::from_entity(entity))
    }

    /// Deletes a ping format field.
    ///
    /// Deletes the field with the specified ID. Associated fleet field values
    /// are automatically deleted due to CASCADE foreign key constraint.
    ///
    /// # Arguments
    /// - `id` - ID of the field to delete
    ///
    /// # Returns
    /// - `Ok(())` - Field deleted successfully (or didn't exist)
    /// - `Err(DbErr)` - Database error during delete operation
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::PingFormatField::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }
}
