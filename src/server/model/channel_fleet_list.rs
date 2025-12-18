//! Domain models for channel fleet list data operations.
//!
//! Defines models for tracking pinned fleet list messages in Discord channels.

use chrono::{DateTime, Utc};

/// Pinned fleet list message in a Discord channel.
///
/// Tracks the channel and message IDs for fleet list messages, along with the last
/// message timestamp to determine whether to edit existing messages or repost new ones.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFleetList {
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

impl ChannelFleetList {
    /// Converts an entity model to a channel fleet list domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `ChannelFleetList` - The converted channel fleet list domain model
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

/// Parameters for upserting a channel fleet list message.
///
/// Creates a new record if none exists for the channel, or updates the existing
/// record with the new message ID and timestamps.
#[derive(Debug, Clone)]
pub struct UpsertChannelFleetListParam {
    /// Discord channel ID where the fleet list is posted.
    pub channel_id: String,
    /// Discord message ID of the fleet list message.
    pub message_id: String,
}
