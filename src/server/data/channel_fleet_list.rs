//! Channel fleet list data repository for database operations.
//!
//! This module provides the `ChannelFleetListRepository` for managing channel fleet list
//! records in the database. Channel fleet lists track the pinned fleet list messages posted
//! in Discord channels that display upcoming fleets. The repository handles upserts and queries
//! with proper conversion between entity models and parameter models at the infrastructure
//! boundary.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

use crate::server::model::channel_fleet_list::{ChannelFleetList, UpsertChannelFleetListParam};

/// Repository providing database operations for channel fleet list management.
///
/// This struct holds a reference to the database connection and provides methods
/// for creating, updating, and querying channel fleet list records that track
/// fleet list messages posted in Discord channels.
pub struct ChannelFleetListRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ChannelFleetListRepository<'a> {
    /// Creates a new ChannelFleetListRepository instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `ChannelFleetListRepository` - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Gets the fleet list message for a channel.
    ///
    /// Queries the database for the fleet list record associated with the specified
    /// Discord channel ID. Returns the record if it exists, which contains the message ID
    /// of the pinned fleet list and timestamps for determining whether to edit or repost.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID as a string slice
    ///
    /// # Returns
    /// - `Ok(Some(ChannelFleetListParam))` - Fleet list record found for the channel
    /// - `Ok(None)` - No fleet list record exists for this channel
    /// - `Err(AppError)` - Database error during query
    pub async fn get_by_channel_id(
        &self,
        channel_id: &str,
    ) -> Result<Option<ChannelFleetList>, DbErr> {
        let entity = entity::prelude::ChannelFleetList::find()
            .filter(entity::channel_fleet_list::Column::ChannelId.eq(channel_id))
            .one(self.db)
            .await?;

        Ok(entity.map(ChannelFleetList::from_entity))
    }

    /// Creates or updates the fleet list message for a channel.
    ///
    /// Performs an upsert operation: if a fleet list record already exists for the channel,
    /// updates the message ID and timestamps; otherwise, creates a new record. The
    /// `last_message_at` timestamp is set to the current time to track when the list was
    /// last posted, and `updated_at` is also updated.
    ///
    /// # Arguments
    /// - `param` - Upsert parameters containing channel_id and message_id
    ///
    /// # Returns
    /// - `Ok(ChannelFleetList)` - The created or updated channel fleet list
    /// - `Err(AppError)` - Database error during upsert operation
    pub async fn upsert(
        &self,
        param: UpsertChannelFleetListParam,
    ) -> Result<ChannelFleetList, DbErr> {
        // Check if record exists
        let existing = self.get_by_channel_id(&param.channel_id).await?;

        let now = Utc::now();

        let entity = if let Some(existing) = existing {
            // Update existing record
            let active: entity::channel_fleet_list::ActiveModel =
                entity::channel_fleet_list::ActiveModel {
                    id: ActiveValue::Set(existing.id),
                    channel_id: ActiveValue::Set(existing.channel_id),
                    message_id: ActiveValue::Set(param.message_id),
                    last_message_at: ActiveValue::Set(now),
                    created_at: ActiveValue::Set(existing.created_at),
                    updated_at: ActiveValue::Set(now),
                };
            active.update(self.db).await?
        } else {
            // Create new record
            let new_record = entity::channel_fleet_list::ActiveModel {
                id: ActiveValue::NotSet,
                channel_id: ActiveValue::Set(param.channel_id),
                message_id: ActiveValue::Set(param.message_id),
                last_message_at: ActiveValue::Set(now),
                created_at: ActiveValue::Set(now),
                updated_at: ActiveValue::Set(now),
            };
            new_record.insert(self.db).await?
        };

        Ok(ChannelFleetList::from_entity(entity))
    }
}
