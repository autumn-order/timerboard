//! Fleet message data repository for database operations.
//!
//! This module provides the `FleetMessageRepository` for managing fleet message records in
//! the database. Fleet messages track Discord messages posted for fleet notifications including
//! creation announcements, reminders, and formup calls. The repository handles creation and
//! queries with proper conversion between entity models and parameter models at the
//! infrastructure boundary.

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

use crate::server::model::fleet_message::{CreateFleetMessageParam, FleetMessage};

/// Repository providing database operations for fleet message management.
///
/// This struct holds a reference to the database connection and provides methods
/// for creating and querying fleet message records that track Discord notifications.
pub struct FleetMessageRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetMessageRepository<'a> {
    /// Creates a new FleetMessageRepository instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `FleetMessageRepository` - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet message record.
    ///
    /// Inserts a new record tracking a Discord message posted for fleet notifications.
    /// Fleet messages can be of different types: creation announcements, reminders before
    /// fleet time, or formup calls at fleet time. These records are used to update or
    /// delete Discord messages when fleet details change.
    ///
    /// # Arguments
    /// - `param` - Create parameters containing fleet_id, channel_id, message_id, and message_type
    ///
    /// # Returns
    /// - `Ok(FleetMessageParam)` - The created fleet message record with generated ID
    /// - `Err(DbErr)` - Database error during insert operation (including foreign key violation)
    pub async fn create(&self, param: CreateFleetMessageParam) -> Result<FleetMessage, DbErr> {
        let entity = entity::fleet_message::ActiveModel {
            fleet_id: ActiveValue::Set(param.fleet_id),
            channel_id: ActiveValue::Set(param.channel_id.to_string()),
            message_id: ActiveValue::Set(param.message_id.to_string()),
            message_type: ActiveValue::Set(param.message_type),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        Ok(FleetMessage::from_entity(entity))
    }

    /// Gets all messages for a fleet.
    ///
    /// Returns all fleet message records for the specified fleet ID. Used to retrieve
    /// Discord message IDs when updating or deleting fleet notifications as fleet
    /// details change or when the fleet is cancelled.
    ///
    /// # Arguments
    /// - `fleet_id` - ID of the fleet to get messages for
    ///
    /// # Returns
    /// - `Ok(Vec<FleetMessage>)` - Vector of fleet messages (empty if no messages)
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_by_fleet_id(&self, fleet_id: i32) -> Result<Vec<FleetMessage>, DbErr> {
        let entities = entity::prelude::FleetMessage::find()
            .filter(entity::fleet_message::Column::FleetId.eq(fleet_id))
            .all(self.db)
            .await?;

        Ok(entities
            .into_iter()
            .map(FleetMessage::from_entity)
            .collect())
    }
}
