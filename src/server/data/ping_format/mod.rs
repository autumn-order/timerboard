//! Ping format data repository for database operations.
//!
//! This module provides the `PingFormatRepository` and `PingFormatFieldRepository` for managing
//! ping format and field records in the database. It handles creation, updates, queries, and
//! deletion with proper conversion between entity models and parameter models at the
//! infrastructure boundary.

pub mod field;

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::server::model::ping_format::{CreatePingFormatParam, PingFormat, UpdatePingFormatParam};

/// Repository providing database operations for ping format management.
///
/// This struct holds a reference to the database connection and provides methods
/// for creating, reading, updating, and deleting ping format records.
pub struct PingFormatRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PingFormatRepository<'a> {
    /// Creates a new PingFormatRepository instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `PingFormatRepository` - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new ping format.
    ///
    /// Inserts a new ping format record into the database with the specified guild ID and name.
    /// The ping format fields must be created separately using `PingFormatFieldRepository`.
    ///
    /// # Arguments
    /// - `param` - Create parameters containing guild_id and name
    ///
    /// # Returns
    /// - `Ok(PingFormat)` - The created ping format with generated ID
    /// - `Err(DbErr)` - Database error during insert operation
    pub async fn create(&self, param: CreatePingFormatParam) -> Result<PingFormat, DbErr> {
        let entity = entity::ping_format::ActiveModel {
            guild_id: ActiveValue::Set(param.guild_id.to_string()),
            name: ActiveValue::Set(param.name),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        Ok(PingFormat::from_entity(entity))
    }

    /// Gets paginated ping formats for a guild.
    ///
    /// Returns a paginated list of ping formats belonging to the specified guild,
    /// ordered alphabetically by name. Used for displaying ping format lists in
    /// guild management interfaces.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID as u64
    /// - `page` - Zero-indexed page number
    /// - `per_page` - Number of ping formats to return per page
    ///
    /// # Returns
    /// - `Ok((formats, total))` - Vector of ping formats and total count
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_all_by_guild_paginated(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<PingFormat>, u64), DbErr> {
        let paginator = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::ping_format::Column::Name)
            .paginate(self.db, per_page);

        let total = paginator.num_items().await?;
        let entities = paginator.fetch_page(page).await?;
        let ping_formats = entities.into_iter().map(PingFormat::from_entity).collect();

        Ok((ping_formats, total))
    }

    /// Updates a ping format's name.
    ///
    /// Updates the name of an existing ping format. Returns the updated ping format
    /// if successful. Fields are managed separately through `PingFormatFieldRepository`.
    ///
    /// # Arguments
    /// - `param` - Update parameters containing id and new name
    ///
    /// # Returns
    /// - `Ok(PingFormat)` - The updated ping format with new name
    /// - `Err(DbErr::RecordNotFound)` - No ping format exists with the specified ID
    /// - `Err(DbErr)` - Other database error during update operation
    pub async fn update(&self, param: UpdatePingFormatParam) -> Result<PingFormat, DbErr> {
        let ping_format = entity::prelude::PingFormat::find_by_id(param.id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Ping format with id {} not found",
                param.id
            )))?;

        let mut active_model: entity::ping_format::ActiveModel = ping_format.into();
        active_model.name = ActiveValue::Set(param.name);

        let entity = active_model.update(self.db).await?;

        Ok(PingFormat::from_entity(entity))
    }

    /// Deletes a ping format.
    ///
    /// Deletes the ping format with the specified ID. Associated ping format fields
    /// are automatically deleted due to CASCADE foreign key constraint. Fleet categories
    /// using this format will have their ping_format_id set to NULL.
    ///
    /// # Arguments
    /// - `id` - ID of the ping format to delete
    ///
    /// # Returns
    /// - `Ok(())` - Ping format deleted successfully (or didn't exist)
    /// - `Err(DbErr)` - Database error during delete operation
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::PingFormat::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// Checks if a ping format exists and belongs to the specified guild.
    ///
    /// Verifies that a ping format with the given ID exists and is owned by the
    /// specified guild. Used for authorization checks before allowing updates or deletions.
    ///
    /// # Arguments
    /// - `id` - ID of the ping format to check
    /// - `guild_id` - Discord guild ID as u64
    ///
    /// # Returns
    /// - `Ok(true)` - Ping format exists and belongs to the guild
    /// - `Ok(false)` - Ping format doesn't exist or belongs to a different guild
    /// - `Err(DbErr)` - Database error during query
    pub async fn exists_in_guild(&self, id: i32, guild_id: u64) -> Result<bool, DbErr> {
        let count = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::Id.eq(id))
            .filter(entity::ping_format::Column::GuildId.eq(guild_id.to_string()))
            .count(self.db)
            .await?;

        Ok(count > 0)
    }

    /// Gets the count of fleet categories using a specific ping format.
    ///
    /// Returns the number of fleet categories that are currently configured to use
    /// the specified ping format. Used to prevent deletion of formats that are in use
    /// and to display usage information to users.
    ///
    /// # Arguments
    /// - `ping_format_id` - ID of the ping format to check
    ///
    /// # Returns
    /// - `Ok(u64)` - Number of fleet categories using this ping format
    /// - `Err(DbErr)` - Database error during count query
    pub async fn get_fleet_category_count(&self, ping_format_id: i32) -> Result<u64, DbErr> {
        entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::PingFormatId.eq(ping_format_id))
            .count(self.db)
            .await
    }
}
