use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;

use crate::{
    model::fleet::{CreateFleetDto, FleetDto, FleetListItemDto, PaginatedFleetsDto},
    server::{data::fleet::FleetRepository, error::AppError},
};

pub struct FleetService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet
    ///
    /// # Arguments
    /// - `dto`: Fleet creation data
    ///
    /// # Returns
    /// - `Ok(FleetDto)`: The created fleet with enriched data
    /// - `Err(AppError)`: Validation or database error
    pub async fn create(&self, dto: CreateFleetDto) -> Result<FleetDto, AppError> {
        let repo = FleetRepository::new(self.db);

        // Parse the fleet time from "YYYY-MM-DD HH:MM" format
        let fleet_time = Self::parse_fleet_time(&dto.fleet_time)?;

        // Create the fleet
        let fleet = repo
            .create(
                dto.category_id,
                dto.name,
                dto.commander_id,
                fleet_time,
                dto.description,
                dto.field_values,
            )
            .await?;

        // Fetch the full fleet data with enriched information
        self.get_by_id(fleet.id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet not found after creation".to_string()))
    }

    /// Gets a fleet by ID with enriched data (category name, commander name, field names)
    ///
    /// # Arguments
    /// - `id`: Fleet ID
    ///
    /// # Returns
    /// - `Ok(Some(FleetDto))`: Fleet found with enriched data
    /// - `Ok(None)`: Fleet not found
    /// - `Err(AppError)`: Database error
    pub async fn get_by_id(&self, id: i32) -> Result<Option<FleetDto>, AppError> {
        let repo = FleetRepository::new(self.db);

        let result = repo.get_by_id(id).await?;

        if let Some((fleet, field_values_by_id)) = result {
            // Fetch category
            let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                .one(self.db)
                .await?
                .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

            // Fetch commander
            let commander = entity::prelude::User::find_by_id(&fleet.commander_id)
                .one(self.db)
                .await?
                .ok_or_else(|| AppError::NotFound("Commander not found".to_string()))?;

            // Fetch field names for the ping format
            let fields = entity::prelude::PingFormatField::find()
                .filter(entity::ping_format_field::Column::PingFormatId.eq(category.ping_format_id))
                .all(self.db)
                .await?;

            let field_name_map: HashMap<i32, String> =
                fields.into_iter().map(|f| (f.id, f.name)).collect();

            // Convert field_values from field_id -> value to field_name -> value
            let field_values: HashMap<String, String> = field_values_by_id
                .into_iter()
                .filter_map(|(field_id, value)| {
                    field_name_map
                        .get(&field_id)
                        .map(|name| (name.clone(), value))
                })
                .collect();

            let commander_id = commander
                .discord_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid commander_id: {}", e)))?;

            Ok(Some(FleetDto {
                id: fleet.id,
                category_id: fleet.category_id,
                category_name: category.name,
                name: fleet.name,
                commander_id,
                commander_name: commander.name,
                fleet_time: fleet.fleet_time,
                description: fleet.description,
                field_values,
                created_at: fleet.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Gets paginated fleets for a guild
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    /// - `page`: Page number (0-indexed)
    /// - `per_page`: Number of items per page
    ///
    /// # Returns
    /// - `Ok(PaginatedFleetsDto)`: Paginated fleet list with enriched data
    /// - `Err(AppError)`: Database error
    pub async fn get_paginated_by_guild(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedFleetsDto, AppError> {
        let repo = FleetRepository::new(self.db);

        let (fleets, total) = repo
            .get_paginated_by_guild(guild_id, page, per_page)
            .await?;

        let total_pages = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as u64
        } else {
            0
        };

        // Enrich fleet data with category and commander names
        let mut fleet_list = Vec::new();

        for fleet in fleets {
            // Fetch category
            let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                .one(self.db)
                .await?;

            // Fetch commander
            let commander = entity::prelude::User::find_by_id(&fleet.commander_id)
                .one(self.db)
                .await?;

            if let (Some(category), Some(commander)) = (category, commander) {
                let commander_id = commander
                    .discord_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid commander_id: {}", e)))?;

                fleet_list.push(FleetListItemDto {
                    id: fleet.id,
                    category_id: fleet.category_id,
                    category_name: category.name,
                    name: fleet.name,
                    commander_id,
                    commander_name: commander.name,
                    fleet_time: fleet.fleet_time,
                });
            }
        }

        Ok(PaginatedFleetsDto {
            fleets: fleet_list,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Gets upcoming fleets for a guild
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    /// - `limit`: Maximum number of fleets to return
    ///
    /// # Returns
    /// - `Ok(Vec<FleetListItemDto>)`: List of upcoming fleets
    /// - `Err(AppError)`: Database error
    pub async fn get_upcoming_by_guild(
        &self,
        guild_id: u64,
        limit: u64,
    ) -> Result<Vec<FleetListItemDto>, AppError> {
        let repo = FleetRepository::new(self.db);

        let fleets = repo.get_upcoming_by_guild(guild_id, limit).await?;

        let mut fleet_list = Vec::new();

        for fleet in fleets {
            // Fetch category
            let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                .one(self.db)
                .await?;

            // Fetch commander
            let commander = entity::prelude::User::find_by_id(&fleet.commander_id)
                .one(self.db)
                .await?;

            if let (Some(category), Some(commander)) = (category, commander) {
                let commander_id = commander
                    .discord_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid commander_id: {}", e)))?;

                fleet_list.push(FleetListItemDto {
                    id: fleet.id,
                    category_id: fleet.category_id,
                    category_name: category.name,
                    name: fleet.name,
                    commander_id,
                    commander_name: commander.name,
                    fleet_time: fleet.fleet_time,
                });
            }
        }

        Ok(fleet_list)
    }

    /// Deletes a fleet
    ///
    /// # Arguments
    /// - `id`: Fleet ID
    /// - `guild_id`: Guild ID (for authorization check)
    ///
    /// # Returns
    /// - `Ok(true)`: Fleet deleted
    /// - `Ok(false)`: Fleet not found or doesn't belong to guild
    /// - `Err(AppError)`: Database error
    pub async fn delete(&self, id: i32, guild_id: u64) -> Result<bool, AppError> {
        let repo = FleetRepository::new(self.db);

        // Check if fleet exists and belongs to the guild
        let result = repo.get_by_id(id).await?;

        if let Some((fleet, _)) = result {
            // Fetch category to verify guild
            let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                .one(self.db)
                .await?;

            if let Some(category) = category {
                let category_guild_id = category
                    .guild_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid guild_id: {}", e)))?;

                if category_guild_id == guild_id {
                    repo.delete(id).await?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Parses fleet time from "YYYY-MM-DD HH:MM" format to DateTime<Utc>
    ///
    /// # Arguments
    /// - `time_str`: Time string in format "YYYY-MM-DD HH:MM"
    ///
    /// # Returns
    /// - `Ok(DateTime<Utc>)`: Parsed datetime
    /// - `Err(AppError)`: Invalid format
    fn parse_fleet_time(time_str: &str) -> Result<DateTime<Utc>, AppError> {
        NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M")
            .map(|naive| naive.and_utc())
            .map_err(|e| {
                AppError::BadRequest(format!(
                    "Invalid fleet time format. Expected 'YYYY-MM-DD HH:MM', got '{}': {}",
                    time_str, e
                ))
            })
    }
}
