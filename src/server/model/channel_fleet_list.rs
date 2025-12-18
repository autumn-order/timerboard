//! Parameter models for channel fleet list data operations.
//!
//! This module defines the parameter models used internally on the server for channel fleet list
//! operations. These models serve as the boundary between the data layer and service/controller
//! layers, with conversion methods to/from entity models.

use chrono::{DateTime, Utc};

/// Represents a channel fleet list with full data from the database.
///
/// Contains all channel fleet list information including IDs, channel/message identifiers,
/// and timestamps. Channel fleet lists track the pinned fleet list messages posted in
/// Discord channels that display upcoming fleets.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFleetListParam {
    /// Unique identifier for the channel fleet list record.
    pub id: i32,
    /// Discord channel ID where the fleet list is posted (stored as String).
    pub channel_id: String,
    /// Discord message ID of the fleet list message (stored as String).
    pub message_id: String,
    /// Timestamp of the last message posted in the channel (for edit vs. repost decision).
    pub last_message_at: DateTime<Utc>,
    /// Timestamp when the record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ChannelFleetListParam {
    /// Converts an entity model to a channel fleet list param.
    ///
    /// This conversion happens at the data layer boundary to ensure entity models
    /// never leak into service or controller layers.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `ChannelFleetListParam` - The converted channel fleet list param
    pub fn from_entity(entity: entity::channel_fleet_list::Model) -> Self {
        Self {
            id: entity.id,
            channel_id: entity.channel_id,
            message_id: entity.message_id,
            last_message_at: entity.last_message_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

/// Parameters for upserting a channel fleet list.
///
/// Used when creating or updating the fleet list message for a channel. The upsert operation
/// will create a new record if none exists, or update the existing record with the new
/// message ID and timestamps.
#[derive(Debug, Clone)]
pub struct UpsertChannelFleetListParam {
    /// Discord channel ID where the fleet list is posted.
    pub channel_id: String,
    /// Discord message ID of the fleet list message.
    pub message_id: String,
}
