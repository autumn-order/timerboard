//! Parameter models for fleet data operations.
//!
//! This module defines the parameter models used internally on the server for fleet
//! operations. These models serve as the boundary between the data layer and service/controller
//! layers, with conversion methods to/from entity models.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Represents a fleet with full data from the database.
///
/// Contains all fleet information including ID, category, commander, timing, and settings.
/// This is the primary model returned by repository methods.
#[derive(Debug, Clone, PartialEq)]
pub struct FleetParam {
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

impl FleetParam {
    /// Converts an entity model to a fleet param.
    ///
    /// This conversion happens at the data layer boundary to ensure entity models
    /// never leak into service or controller layers.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `FleetParam` - The converted fleet param
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

/// Parameters for creating a new fleet.
///
/// Used when creating a new fleet operation with initial configuration and field values.
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

/// Parameters for updating an existing fleet.
///
/// Used when updating fleet details. All fields are optional - only provided fields
/// will be updated. Field values can be completely replaced if provided.
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
