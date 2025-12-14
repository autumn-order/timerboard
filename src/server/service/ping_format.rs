use sea_orm::DatabaseConnection;

use crate::{
    model::ping_format::{PaginatedPingFormatsDto, PingFormatDto, PingFormatFieldDto},
    server::{
        data::ping_format::{PingFormatFieldRepository, PingFormatRepository},
        error::AppError,
    },
};

pub struct PingFormatService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PingFormatService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new ping format with its fields
    pub async fn create(
        &self,
        guild_id: u64,
        name: String,
        fields: Vec<(String, i32)>, // (name, priority)
    ) -> Result<PingFormatDto, AppError> {
        let format_repo = PingFormatRepository::new(self.db);
        let field_repo = PingFormatFieldRepository::new(self.db);

        // Create the ping format
        let ping_format = format_repo.create(guild_id, name).await?;

        // Create all the fields
        let mut result_fields = Vec::new();
        for (field_name, priority) in fields {
            let field = field_repo
                .create(ping_format.id, field_name, priority)
                .await?;
            result_fields.push(PingFormatFieldDto {
                id: field.id,
                ping_format_id: field.ping_format_id,
                name: field.name,
                priority: field.priority,
            });
        }

        // Get fleet category count
        let fleet_category_count = format_repo.get_fleet_category_count(ping_format.id).await?;

        let guild_id = ping_format
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Failed to parse guild_id: {}", e)))?;

        Ok(PingFormatDto {
            id: ping_format.id,
            guild_id,
            name: ping_format.name,
            fields: result_fields,
            fleet_category_count,
        })
    }

    /// Gets paginated ping formats for a guild with all their fields
    pub async fn get_paginated(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedPingFormatsDto, AppError> {
        let format_repo = PingFormatRepository::new(self.db);
        let field_repo = PingFormatFieldRepository::new(self.db);

        let (ping_formats, total) = format_repo
            .get_by_guild_id_paginated(guild_id, page, per_page)
            .await?;

        let total_pages = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as u64
        } else {
            0
        };

        let mut ping_format_dtos = Vec::new();
        for ping_format in ping_formats {
            let fields = field_repo.get_by_ping_format_id(ping_format.id).await?;

            let fleet_category_count = format_repo.get_fleet_category_count(ping_format.id).await?;

            let guild_id = ping_format
                .guild_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Failed to parse guild_id: {}", e)))?;

            ping_format_dtos.push(PingFormatDto {
                id: ping_format.id,
                guild_id,
                name: ping_format.name,
                fields: fields
                    .into_iter()
                    .map(|f| PingFormatFieldDto {
                        id: f.id,
                        ping_format_id: f.ping_format_id,
                        name: f.name,
                        priority: f.priority,
                    })
                    .collect(),
                fleet_category_count,
            });
        }

        Ok(PaginatedPingFormatsDto {
            ping_formats: ping_format_dtos,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Updates a ping format's name and fields
    /// Returns None if the ping format doesn't exist or doesn't belong to the guild
    pub async fn update(
        &self,
        id: i32,
        guild_id: u64,
        name: String,
        fields: Vec<(Option<i32>, String, i32)>, // (id, name, priority) - id is None for new fields
    ) -> Result<Option<PingFormatDto>, AppError> {
        let format_repo = PingFormatRepository::new(self.db);
        let field_repo = PingFormatFieldRepository::new(self.db);

        // Check if ping format exists and belongs to the guild
        if !format_repo.exists_in_guild(id, guild_id).await? {
            return Ok(None);
        }

        // Update the ping format
        let ping_format = format_repo.update(id, name).await?;

        // Get existing fields
        let existing_fields = field_repo.get_by_ping_format_id(ping_format.id).await?;

        // Determine which fields to keep, update, or create
        let mut updated_fields = Vec::new();
        let mut existing_field_ids: Vec<i32> = Vec::new();

        for (field_id, field_name, priority) in fields {
            if let Some(id) = field_id {
                // Update existing field
                let field = field_repo.update(id, field_name, priority).await?;
                existing_field_ids.push(id);
                updated_fields.push(PingFormatFieldDto {
                    id: field.id,
                    ping_format_id: field.ping_format_id,
                    name: field.name,
                    priority: field.priority,
                });
            } else {
                // Create new field
                let field = field_repo
                    .create(ping_format.id, field_name, priority)
                    .await?;
                updated_fields.push(PingFormatFieldDto {
                    id: field.id,
                    ping_format_id: field.ping_format_id,
                    name: field.name,
                    priority: field.priority,
                });
            }
        }

        // Delete fields that are no longer present
        for existing_field in existing_fields {
            if !existing_field_ids.contains(&existing_field.id) {
                field_repo.delete(existing_field.id).await?;
            }
        }

        // Get fleet category count
        let fleet_category_count = format_repo.get_fleet_category_count(ping_format.id).await?;

        let guild_id = ping_format
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Some(PingFormatDto {
            id: ping_format.id,
            guild_id,
            name: ping_format.name,
            fields: updated_fields,
            fleet_category_count,
        }))
    }

    /// Deletes a ping format and all its fields
    /// Returns true if deleted, false if not found or doesn't belong to guild
    pub async fn delete(&self, id: i32, guild_id: u64) -> Result<bool, AppError> {
        let format_repo = PingFormatRepository::new(self.db);

        // Check if ping format exists and belongs to the guild
        if !format_repo.exists_in_guild(id, guild_id).await? {
            return Ok(false);
        }

        // Delete the ping format (fields will be deleted by cascade)
        format_repo.delete(id).await?;

        Ok(true)
    }
}
