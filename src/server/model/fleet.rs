//! Domain models for fleet data operations.
//!
//! Defines fleet-related domain models and parameter types for fleet operations.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Fleet operation with scheduling, commander, and configuration details.
///
/// Tracks fleet category, commander, scheduled time, visibility settings, and
/// reminder preferences for organized fleet operations.
#[derive(Debug, Clone, PartialEq)]
pub struct Fleet {
    /// Unique identifier for the fleet.
    pub id: i32,
    /// ID of the fleet category this fleet belongs to.
    pub category_id: i32,
    /// Name of the fleet operation.
    pub name: String,
    /// Discord ID of the fleet commander (stored as String).
    pub commander_id: String,
    /// Scheduled time for the fleet operation.
    pub fleet_time: DateTime<Utc>,
    /// Optional description of the fleet operation.
    pub description: Option<String>,
    /// Whether the fleet is hidden from non-privileged users.
    pub hidden: bool,
    /// Whether reminder notifications are disabled for this fleet.
    pub disable_reminder: bool,
    /// Timestamp when the fleet was created.
    pub created_at: DateTime<Utc>,
}

impl Fleet {
    /// Converts an entity model to a fleet domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `Fleet` - The converted fleet domain model
    pub fn from_entity(entity: entity::fleet::Model) -> Self {
        Self {
            id: entity.id,
            category_id: entity.category_id,
            name: entity.name,
            commander_id: entity.commander_id,
            fleet_time: entity.fleet_time,
            description: entity.description,
            hidden: entity.hidden,
            disable_reminder: entity.disable_reminder,
            created_at: entity.created_at,
        }
    }
}

/// Parameters for creating a new fleet operation.
///
/// Includes initial configuration, custom field values for the ping format,
/// and visibility/reminder settings.
#[derive(Debug, Clone)]
pub struct CreateFleetParams {
    /// ID of the fleet category this fleet belongs to.
    pub category_id: i32,
    /// Name of the fleet operation.
    pub name: String,
    /// Discord ID of the fleet commander as u64.
    pub commander_id: u64,
    /// Scheduled time for the fleet operation.
    pub fleet_time: DateTime<Utc>,
    /// Optional description of the fleet operation.
    pub description: Option<String>,
    /// Map of field_id to field value for custom ping format fields.
    pub field_values: HashMap<i32, String>,
    /// Whether the fleet should be hidden from non-privileged users.
    pub hidden: bool,
    /// Whether reminder notifications should be disabled for this fleet.
    pub disable_reminder: bool,
}

/// Parameters for updating an existing fleet operation.
///
/// All fields are optional - only provided fields will be updated. The field_values
/// map, if provided, completely replaces all existing field values.
#[derive(Debug, Clone)]
pub struct UpdateFleetParams {
    /// ID of the fleet to update.
    pub id: i32,
    /// New fleet category ID if changing categories.
    pub category_id: Option<i32>,
    /// New name for the fleet operation.
    pub name: Option<String>,
    /// New scheduled time for the fleet operation.
    pub fleet_time: Option<DateTime<Utc>>,
    /// New description (outer Option indicates field presence, inner for nullable value).
    pub description: Option<Option<String>>,
    /// New field values (replaces all existing field values if provided).
    pub field_values: Option<HashMap<i32, String>>,
    /// New hidden status.
    pub hidden: Option<bool>,
    /// New disable_reminder status.
    pub disable_reminder: Option<bool>,
}

/// Parameters for retrieving paginated fleets for a guild.
///
/// Includes guild and user context for permission filtering, admin status for
/// bypassing restrictions, and pagination configuration.
#[derive(Debug, Clone)]
pub struct GetPaginatedFleetsByGuildParam {
    /// Discord guild ID to fetch fleets for.
    pub guild_id: u64,
    /// Discord user ID for permission filtering.
    pub user_id: u64,
    /// Whether the user is an admin (bypasses all filtering).
    pub is_admin: bool,
    /// Page number (0-indexed).
    pub page: u64,
    /// Number of items per page.
    pub per_page: u64,
}
