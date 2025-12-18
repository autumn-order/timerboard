//! Parameter models for ping format data operations.
//!
//! This module defines the parameter models used internally on the server for ping format
//! and ping format field operations. These models serve as the boundary between the data
//! layer and service/controller layers, with conversion methods to/from entity models and DTOs.

use crate::model::ping_format::{PingFormatDto, PingFormatFieldDto};

/// Represents a ping format with full data from the database.
///
/// Contains all ping format information including ID, guild ID, and name.
/// This is the primary model returned by repository methods.
#[derive(Debug, Clone, PartialEq)]
pub struct PingFormatParam {
    /// Unique identifier for the ping format.
    pub id: i32,
    /// Discord guild ID (stored as String in database).
    pub guild_id: String,
    /// Name of the ping format.
    pub name: String,
}

impl PingFormatParam {
    /// Converts the ping format param to a DTO for API responses.
    ///
    /// Note: This conversion only includes basic fields. Additional data like fields,
    /// fleet_category_count, and fleet_category_names must be provided separately.
    ///
    /// # Arguments
    /// - `self`: The ping format param to convert
    /// - `fields`: Vector of ping format field DTOs
    /// - `fleet_category_count`: Number of fleet categories using this format
    /// - `fleet_category_names`: Names of fleet categories using this format
    ///
    /// # Returns
    /// - `PingFormatDto`: The converted ping format DTO with guild_id as u64
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

    /// Converts an entity model to a ping format param.
    ///
    /// This conversion happens at the data layer boundary to ensure entity models
    /// never leak into service or controller layers.
    ///
    /// # Arguments
    /// - `entity`: The entity model from the database
    ///
    /// # Returns
    /// - `PingFormatParam`: The converted ping format param
    pub fn from_entity(entity: entity::ping_format::Model) -> Self {
        Self {
            id: entity.id,
            guild_id: entity.guild_id,
            name: entity.name,
        }
    }
}

/// Parameters for creating a new ping format.
///
/// Used when creating a new ping format in the database with initial field configuration.
#[derive(Debug, Clone)]
pub struct CreatePingFormatParam {
    /// Discord guild ID as u64.
    pub guild_id: u64,
    /// Name of the ping format.
    pub name: String,
}

/// Parameters for updating an existing ping format.
///
/// Used when updating a ping format's name. Fields are managed separately.
#[derive(Debug, Clone)]
pub struct UpdatePingFormatParam {
    /// ID of the ping format to update.
    pub id: i32,
    /// New name for the ping format.
    pub name: String,
}

/// Represents a ping format field with full data from the database.
///
/// Contains all field information including ID, ping format ID, name, priority,
/// and default value. Fields define the structure of ping messages.
#[derive(Debug, Clone, PartialEq)]
pub struct PingFormatFieldParam {
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

impl PingFormatFieldParam {
    /// Converts the ping format field param to a DTO for API responses.
    ///
    /// # Arguments
    /// - `self`: The ping format field param to convert
    ///
    /// # Returns
    /// - `PingFormatFieldDto`: The converted field DTO
    pub fn into_dto(self) -> PingFormatFieldDto {
        PingFormatFieldDto {
            id: self.id,
            ping_format_id: self.ping_format_id,
            name: self.name,
            priority: self.priority,
            default_value: self.default_value,
        }
    }

    /// Converts an entity model to a ping format field param.
    ///
    /// This conversion happens at the data layer boundary to ensure entity models
    /// never leak into service or controller layers.
    ///
    /// # Arguments
    /// - `entity`: The entity model from the database
    ///
    /// # Returns
    /// - `PingFormatFieldParam`: The converted field param
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

/// Parameters for creating a new ping format field.
///
/// Used when creating a new field for an existing ping format.
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

/// Parameters for updating an existing ping format field.
///
/// Used when updating a field's name, priority, or default value.
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
