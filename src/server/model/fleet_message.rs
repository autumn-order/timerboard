//! Parameter models for fleet message data operations.
//!
//! This module defines the parameter models used internally on the server for fleet message
//! operations. These models serve as the boundary between the data layer and service/controller
//! layers, with conversion methods to/from entity models.

use chrono::{DateTime, Utc};

/// Represents a fleet message with full data from the database.
///
/// Contains all fleet message information including IDs, channel/message identifiers,
/// message type, and creation timestamp. Fleet messages track Discord messages posted
/// for fleet notifications (creation, reminders, formup).
#[derive(Debug, Clone, PartialEq)]
pub struct FleetMessageParam {
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

impl FleetMessageParam {
    /// Converts an entity model to a fleet message param.
    ///
    /// This conversion happens at the data layer boundary to ensure entity models
    /// never leak into service or controller layers.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `FleetMessageParam` - The converted fleet message param
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

/// Parameters for creating a new fleet message.
///
/// Used when recording a new Discord message posted for fleet notifications.
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
