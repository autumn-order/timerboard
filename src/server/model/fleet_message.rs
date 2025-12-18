//! Domain models for fleet message data operations.
//!
//! Defines models for tracking Discord messages posted for fleet notifications.

use chrono::{DateTime, Utc};

/// Discord message posted for fleet notifications.
///
/// Tracks messages posted for fleet events such as creation announcements, reminders,
/// and formup notifications. Stores channel and message IDs for message management.
#[derive(Debug, Clone, PartialEq)]
pub struct FleetMessage {
    /// Unique identifier for the fleet message record.
    pub id: i32,
    /// ID of the fleet this message belongs to.
    pub fleet_id: i32,
    /// Discord channel ID where the message was posted (stored as String).
    pub channel_id: String,
    /// Discord message ID (stored as String).
    pub message_id: String,
    /// Type of message (e.g., "creation", "reminder", "formup").
    pub message_type: String,
    /// Timestamp when the message record was created.
    pub created_at: DateTime<Utc>,
}

impl FleetMessage {
    /// Converts an entity model to a fleet message domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `FleetMessage` - The converted fleet message domain model
    pub fn from_entity(entity: entity::fleet_message::Model) -> Self {
        Self {
            id: entity.id,
            fleet_id: entity.fleet_id,
            channel_id: entity.channel_id,
            message_id: entity.message_id,
            message_type: entity.message_type,
            created_at: entity.created_at,
        }
    }
}

/// Parameters for creating a new fleet message record.
///
/// Records a Discord message posted for fleet notifications (creation, reminder, or formup).
#[derive(Debug, Clone)]
pub struct CreateFleetMessageParam {
    /// ID of the fleet this message belongs to.
    pub fleet_id: i32,
    /// Discord channel ID where the message was posted.
    pub channel_id: u64,
    /// Discord message ID.
    pub message_id: u64,
    /// Type of message (e.g., "creation", "reminder", "formup").
    pub message_type: String,
}
