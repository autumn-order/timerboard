//! Domain models for ping format data operations.
//!
//! Defines ping format and field models that structure fleet notification messages.

use crate::{
    model::ping_format::{PaginatedPingFormatsDto, PingFormatDto, PingFormatFieldDto},
    server::{error::AppError, util::parse::parse_u64_from_string},
};

/// Ping format template for structuring fleet notification messages.
///
/// Defines the overall format with a name and guild association. Contains multiple
/// fields that structure the data displayed in fleet pings.
#[derive(Debug, Clone, PartialEq)]
pub struct PingFormat {
    /// Unique identifier for the ping format.
    pub id: i32,
    /// Discord guild ID.
    pub guild_id: u64,
    /// Name of the ping format.
    pub name: String,
}

impl PingFormat {
    /// Converts an entity model to a ping format domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `Ok(PingFormat)` - The converted ping format domain model
    /// - `Err(AppError::ParseStringId)` - Failed to parse guild ID to u64
    pub fn from_entity(entity: entity::ping_format::Model) -> Result<Self, AppError> {
        let guild_id = parse_u64_from_string(entity.guild_id)?;

        Ok(Self {
            id: entity.id,
            guild_id,
            name: entity.name,
        })
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

/// Complete ping format with fields and usage metadata.
///
/// Combines the ping format with its fields and information about which
/// fleet categories are using this format. Used for service layer operations
/// that need the complete format data.
#[derive(Debug, Clone, PartialEq)]
pub struct PingFormatWithFields {
    /// The ping format.
    pub ping_format: PingFormat,
    /// Fields belonging to this ping format.
    pub fields: Vec<PingFormatField>,
    /// Number of fleet categories using this format.
    pub fleet_category_count: u64,
    /// Names of fleet categories using this format.
    pub fleet_category_names: Vec<String>,
}

impl PingFormatWithFields {
    /// Converts the complete ping format to a DTO for API responses.
    ///
    /// Parses the stored String guild_id into u64 for the DTO. If parsing fails,
    /// returns an error.
    ///
    /// # Returns
    /// - `PingFormatDto` - Ping format DTO for API responses
    pub fn into_dto(self) -> PingFormatDto {
        let field_dtos = self.fields.into_iter().map(|f| f.into_dto()).collect();

        PingFormatDto {
            id: self.ping_format.id,
            guild_id: self.ping_format.guild_id,
            name: self.ping_format.name,
            fields: field_dtos,
            fleet_category_count: self.fleet_category_count,
            fleet_category_names: self.fleet_category_names,
        }
    }
}

/// Paginated collection of ping formats with metadata.
///
/// Contains a page of ping formats along with pagination metadata for building
/// paginated ping format management interfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct PaginatedPingFormats {
    /// Ping formats for this page.
    pub ping_formats: Vec<PingFormatWithFields>,
    /// Total number of ping formats across all pages.
    pub total: u64,
    /// Current page number (zero-indexed).
    pub page: u64,
    /// Number of ping formats per page.
    pub per_page: u64,
    /// Total number of pages.
    pub total_pages: u64,
}

impl PaginatedPingFormats {
    /// Converts the paginated ping formats to a DTO for API responses.
    ///
    /// Converts each ping format in the collection to a DTO. If any conversion fails,
    /// returns an error immediately without processing remaining formats.
    ///
    /// # Returns
    /// - `Ok(PaginatedPingFormatsDto)` - Successfully converted all formats
    /// - `Err(AppError::ParseStringId)` - Failed to parse guild_id for at least one format
    pub fn into_dto(self) -> PaginatedPingFormatsDto {
        let ping_formats: Vec<PingFormatDto> = self
            .ping_formats
            .into_iter()
            .map(|pf| pf.into_dto())
            .collect();

        PaginatedPingFormatsDto {
            ping_formats,
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}

/// Parameters for creating a ping format with fields.
///
/// Contains all data needed to create a new ping format along with its fields
/// in a single operation.
#[derive(Debug, Clone)]
pub struct CreatePingFormatWithFieldsParam {
    /// Discord guild ID.
    pub guild_id: u64,
    /// Name of the ping format.
    pub name: String,
    /// Fields to create (name, priority, default_value).
    pub fields: Vec<(String, i32, Option<String>)>,
}

/// Parameters for updating a ping format with fields.
///
/// Contains all data needed to update a ping format along with its fields.
/// Fields with an id will be updated, fields without an id will be created,
/// and existing fields not in the list will be deleted.
#[derive(Debug, Clone)]
pub struct UpdatePingFormatWithFieldsParam {
    /// ID of the ping format to update.
    pub id: i32,
    /// Discord guild ID for verification.
    pub guild_id: u64,
    /// New name for the ping format.
    pub name: String,
    /// Fields to update/create (id, name, priority, default_value) - id is None for new fields.
    pub fields: Vec<(Option<i32>, String, i32, Option<String>)>,
}

/// Parameters for getting paginated ping formats.
#[derive(Debug, Clone)]
pub struct GetPaginatedPingFormatsParam {
    /// Discord guild ID to filter by.
    pub guild_id: u64,
    /// Zero-indexed page number.
    pub page: u64,
    /// Number of ping formats per page.
    pub per_page: u64,
}

/// Parameters for deleting a ping format.
#[derive(Debug, Clone)]
pub struct DeletePingFormatParam {
    /// ID of the ping format to delete.
    pub id: i32,
    /// Discord guild ID for verification.
    pub guild_id: u64,
}
