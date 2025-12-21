//! Domain models for channel fleet list data operations.
//!
//! Defines models for tracking pinned fleet list messages in Discord channels.

use chrono::{DateTime, Utc};

use crate::server::{error::AppError, util::parse::parse_u64_from_string};

/// Pinned fleet list message in a Discord channel.
///
/// Tracks the channel and message IDs for fleet list messages, along with the last
/// message timestamp to determine whether to edit existing messages or repost new ones.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFleetList {
    /// Unique identifier for the channel fleet list record.
    pub id: i32,
    /// Discord channel ID where the fleet list is posted (stored as String).
    pub channel_id: u64,
    /// Discord message ID of the fleet list message (stored as String).
    pub message_id: u64,
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
    /// - `Ok(ChannelFleetList)` - The converted channel fleet list domain model
    /// - `Err(AppError::ParseStringId)` - Failed to parse Discord channel or message ID stored as
    ///   string to u64
    pub fn from_entity(entity: entity::channel_fleet_list::Model) -> Result<Self, AppError> {
        let channel_id = parse_u64_from_string(entity.channel_id)?;
        let message_id = parse_u64_from_string(entity.message_id)?;

        Ok(Self {
            id: entity.id,
            channel_id: channel_id,
            message_id: message_id,
            last_message_at: entity.last_message_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }
}

/// Parameters for upserting a channel fleet list message.
///
/// Creates a new record if none exists for the channel, or updates the existing
/// record with the new message ID and timestamps.
#[derive(Debug, Clone)]
pub struct UpsertChannelFleetListParam {
    /// Discord channel ID where the fleet list is posted.
    pub channel_id: u64,
    /// Discord message ID of the fleet list message.
    pub message_id: u64,
}
