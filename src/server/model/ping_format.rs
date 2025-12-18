//! Domain models for ping format data operations.
//!
//! Defines ping format and field models that structure fleet notification messages.

use crate::model::ping_format::{PingFormatDto, PingFormatFieldDto};

/// Ping format template for structuring fleet notification messages.
///
/// Defines the overall format with a name and guild association. Contains multiple
/// fields that structure the data displayed in fleet pings.
#[derive(Debug, Clone, PartialEq)]
pub struct PingFormat {
    /// Unique identifier for the ping format.
    pub id: i32,
    /// Discord guild ID (stored as String in database).
    pub guild_id: String,
    /// Name of the ping format.
    pub name: String,
}

impl PingFormat {
    /// Converts the ping format domain model to a DTO for API responses.
    ///
    /// # Arguments
    /// - `fields` - Vector of ping format field DTOs
    /// - `fleet_category_count` - Number of fleet categories using this format
    /// - `fleet_category_names` - Names of fleet categories using this format
    ///
    /// # Returns
    /// - `PingFormatDto` - The converted ping format DTO with guild_id as u64
    pub fn into_dto(
        self,
        fields: Vec<PingFormatFieldDto>,
        fleet_category_count: u64,
        fleet_category_names: Vec<String>,
    ) -> PingFormatDto {
        PingFormatDto {
            id: self.id,
            guild_id: self.guild_id.parse().unwrap_or(0),
            name: self.name,
            fields,
            fleet_category_count,
            fleet_category_names,
        }
    }

    /// Converts an entity model to a ping format domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `PingFormat` - The converted ping format domain model
    pub fn from_entity(entity: entity::ping_format::Model) -> Self {
        Self {
            id: entity.id,
            guild_id: entity.guild_id,
            name: entity.name,
        }
    }
}

/// Parameters for creating a new ping format template.
///
/// Creates a new ping format with a name. Fields are added separately after creation.
#[derive(Debug, Clone)]
pub struct CreatePingFormatParam {
    /// Discord guild ID as u64.
    pub guild_id: u64,
    /// Name of the ping format.
    pub name: String,
}

/// Parameters for updating an existing ping format template.
///
/// Updates only the ping format name. Fields are managed through separate operations.
#[derive(Debug, Clone)]
pub struct UpdatePingFormatParam {
    /// ID of the ping format to update.
    pub id: i32,
    /// New name for the ping format.
    pub name: String,
}

/// Individual field within a ping format template.
///
/// Defines a single data field in fleet ping messages with a name, display priority,
/// and optional default value. Lower priority values are displayed first.
#[derive(Debug, Clone, PartialEq)]
pub struct PingFormatField {
    /// Unique identifier for the field.
    pub id: i32,
    /// ID of the ping format this field belongs to.
    pub ping_format_id: i32,
    /// Name of the field.
    pub name: String,
    /// Priority for field ordering (lower values appear first).
    pub priority: i32,
    /// Optional default value for the field.
    pub default_value: Option<String>,
}

impl PingFormatField {
    /// Converts the ping format field domain model to a DTO for API responses.
    ///
    /// # Returns
    /// - `PingFormatFieldDto` - The converted field DTO
    pub fn into_dto(self) -> PingFormatFieldDto {
        PingFormatFieldDto {
            id: self.id,
            ping_format_id: self.ping_format_id,
            name: self.name,
            priority: self.priority,
            default_value: self.default_value,
        }
    }

    /// Converts an entity model to a ping format field domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `PingFormatField` - The converted field domain model
    pub fn from_entity(entity: entity::ping_format_field::Model) -> Self {
        Self {
            id: entity.id,
            ping_format_id: entity.ping_format_id,
            name: entity.name,
            priority: entity.priority,
            default_value: entity.default_value,
        }
    }
}

/// Parameters for creating a new field in a ping format template.
#[derive(Debug, Clone)]
pub struct CreatePingFormatFieldParam {
    /// ID of the ping format this field belongs to.
    pub ping_format_id: i32,
    /// Name of the field.
    pub name: String,
    /// Priority for field ordering.
    pub priority: i32,
    /// Optional default value for the field.
    pub default_value: Option<String>,
}

/// Parameters for updating an existing ping format field's properties.
#[derive(Debug, Clone)]
pub struct UpdatePingFormatFieldParam {
    /// ID of the field to update.
    pub id: i32,
    /// New name for the field.
    pub name: String,
    /// New priority for the field.
    pub priority: i32,
    /// New default value for the field.
    pub default_value: Option<String>,
}
