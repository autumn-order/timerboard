use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;

use crate::{
    model::fleet::{
        CreateFleetDto, FleetDto, FleetListItemDto, PaginatedFleetsDto, UpdateFleetDto,
    },
    server::{
        data::{category::FleetCategoryRepository, fleet::FleetRepository},
        error::AppError,
    },
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
    /// Filters fleets to only include:
    /// - Fleets in categories the user can view (admins can view all)
    /// - Fleets that are not older than 1 hour from the current time
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    /// - `user_id`: Discord user ID for permission filtering
    /// - `is_admin`: Whether the user is an admin (bypasses category filtering)
    /// - `page`: Page number (0-indexed)
    /// - `per_page`: Number of items per page
    ///
    /// # Returns
    /// - `Ok(PaginatedFleetsDto)`: Paginated fleet list with enriched data
    /// - `Err(AppError)`: Database error
    pub async fn get_paginated_by_guild(
        &self,
        guild_id: u64,
        user_id: u64,
        is_admin: bool,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedFleetsDto, AppError> {
        let repo = FleetRepository::new(self.db);

        // Get viewable category IDs for non-admin users
        let viewable_category_ids = if is_admin {
            None // Admins can view all categories
        } else {
            let category_repo = FleetCategoryRepository::new(self.db);
            Some(
                category_repo
                    .get_viewable_category_ids_by_user(user_id, guild_id)
                    .await?,
            )
        };

        let (fleets, total) = repo
            .get_paginated_by_guild(guild_id, page, per_page, viewable_category_ids)
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

    /// Updates a fleet
    ///
    /// # Arguments
    /// - `id`: Fleet ID
    /// - `guild_id`: Guild ID (for authorization check)
    /// - `dto`: Update data
    ///
    /// # Returns
    /// - `Ok(FleetDto)`: The updated fleet with enriched data
    /// - `Err(AppError)`: Validation, authorization, or database error
    pub async fn update(
        &self,
        id: i32,
        guild_id: u64,
        dto: UpdateFleetDto,
    ) -> Result<FleetDto, AppError> {
        let repo = FleetRepository::new(self.db);

        // Get the current fleet to verify it belongs to the guild and get original time
        let result = repo.get_by_id(id).await?;
        if let Some((fleet, _)) = result {
            // Parse the fleet time with original time for validation
            let original_time = fleet.fleet_time;
            let fleet_time = Self::parse_fleet_time_with_min(&dto.fleet_time, Some(original_time))?;
            // Fetch old category to verify guild
            let old_category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                .one(self.db)
                .await?;

            if let Some(old_category) = old_category {
                let category_guild_id = old_category
                    .guild_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid guild_id: {}", e)))?;

                if category_guild_id != guild_id {
                    return Err(AppError::NotFound("Fleet not found".to_string()));
                }

                // If category is being changed, validate the new category belongs to the same guild
                if dto.category_id != fleet.category_id {
                    let new_category = entity::prelude::FleetCategory::find_by_id(dto.category_id)
                        .one(self.db)
                        .await?
                        .ok_or_else(|| AppError::NotFound("New category not found".to_string()))?;

                    let new_category_guild_id = new_category
                        .guild_id
                        .parse::<u64>()
                        .map_err(|e| AppError::InternalError(format!("Invalid guild_id: {}", e)))?;

                    if new_category_guild_id != guild_id {
                        return Err(AppError::BadRequest(
                            "New category does not belong to this guild".to_string(),
                        ));
                    }
                }

                // Update the fleet
                repo.update(
                    id,
                    Some(dto.category_id),
                    Some(dto.name),
                    Some(fleet_time),
                    Some(dto.description),
                    Some(dto.field_values),
                )
                .await?;

                // Fetch the updated fleet data with enriched information
                return self
                    .get_by_id(id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("Fleet not found after update".to_string()));
            }
        }

        Err(AppError::NotFound("Fleet not found".to_string()))
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

    /// Parses fleet time from "YYYY-MM-DD HH:MM" format or "now" to DateTime<Utc>
    ///
    /// Validates that the fleet time is not in the past.
    ///
    /// # Arguments
    /// - `time_str`: Time string in format "YYYY-MM-DD HH:MM" or "now" (case-insensitive)
    ///
    /// # Returns
    /// - `Ok(DateTime<Utc>)`: Parsed datetime
    /// - `Err(AppError)`: Invalid format or time is in the past
    fn parse_fleet_time(time_str: &str) -> Result<DateTime<Utc>, AppError> {
        Self::parse_fleet_time_with_min(time_str, None)
    }

    /// Parse fleet time with optional minimum time for edit validation
    ///
    /// # Arguments
    /// - `time_str`: Time string in format "YYYY-MM-DD HH:MM" or "now"
    /// - `min_time`: Optional minimum time (for edits where original time is in the past)
    ///
    /// # Returns
    /// - `Ok(DateTime<Utc>)`: Parsed fleet time
    /// - `Err(AppError)`: Invalid format or time validation failure
    fn parse_fleet_time_with_min(
        time_str: &str,
        min_time: Option<DateTime<Utc>>,
    ) -> Result<DateTime<Utc>, AppError> {
        let now = Utc::now();

        // Handle "now" shorthand (case-insensitive)
        let fleet_time = if time_str.trim().eq_ignore_ascii_case("now") {
            now
        } else {
            NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M")
                .map(|naive| naive.and_utc())
                .map_err(|e| {
                    AppError::BadRequest(format!(
                        "Invalid fleet time format. Expected 'YYYY-MM-DD HH:MM' or 'now', got '{}': {}",
                        time_str, e
                    ))
                })?
        };

        // If min_time is provided and is in the past, validate against min_time
        if let Some(min_time) = min_time {
            if min_time < now && fleet_time < min_time {
                return Err(AppError::BadRequest(format!(
                    "Fleet time cannot be set earlier than the original time ({})",
                    min_time.format("%Y-%m-%d %H:%M UTC")
                )));
            }
        }

        // Validate fleet time is not in the past (only if min_time is not provided or is in the future)
        if min_time.is_none() || min_time.map(|t| t >= now).unwrap_or(true) {
            if fleet_time < now {
                return Err(AppError::BadRequest(
                    "Fleet time cannot be in the past".to_string(),
                ));
            }
        }

        Ok(fleet_time)
    }
}
