//! Fleet service for managing fleet operations.
//!
//! This module provides the `FleetService` for handling fleet CRUD operations,
//! including creation, retrieval, updates, and deletion. It orchestrates between
//! the data layer, permission checks, Discord notifications, and fleet visibility rules.
//!
//! Fleet visibility is governed by category permissions and hidden fleet rules:
//! - Users must have at least view permission for a category to see its fleets
//! - Hidden fleets are only visible to users with create/manage permissions OR
//!   after the reminder time has elapsed (or fleet start time if no reminder)
//! - Admins bypass all visibility restrictions

use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::http::Http;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    model::fleet::{
        CreateFleetDto, FleetDto, FleetListItemDto, PaginatedFleetsDto, UpdateFleetDto,
    },
    server::{
        data::{category::FleetCategoryRepository, fleet::FleetRepository},
        error::AppError,
        model::fleet::{CreateFleetParams, GetPaginatedFleetsByGuildParam, UpdateFleetParams},
        service::fleet_notification::FleetNotificationService,
    },
};

/// Service for managing fleet operations.
///
/// Handles fleet creation, retrieval, updates, and deletion with integrated
/// permission checks, visibility rules, and Discord notification orchestration.
pub struct FleetService<'a> {
    /// Database connection for fleet operations.
    db: &'a DatabaseConnection,
    /// Discord HTTP client for notification operations.
    discord_http: Arc<Http>,
    /// Base application URL for embedding links in notifications.
    app_url: String,
}

impl<'a> FleetService<'a> {
    pub fn new(db: &'a DatabaseConnection, discord_http: Arc<Http>, app_url: String) -> Self {
        Self {
            db,
            discord_http,
            app_url,
        }
    }

    /// Creates a new fleet with time validation and Discord notifications.
    ///
    /// Creates a fleet after validating the time is not in the past (with 2-minute grace
    /// period) and checking for conflicts with category cooldown settings. Posts a creation
    /// notification to Discord and returns the enriched fleet data.
    ///
    /// # Arguments
    /// - `dto` - Fleet creation data including category, time, commander, and field values
    /// - `is_admin` - Whether the creating user is an admin for permission checks on result
    ///
    /// # Returns
    /// - `Ok(FleetDto)` - Created fleet with enriched data (category name, commander name)
    /// - `Err(AppError::BadRequest(_))` - Time validation failed or conflict with cooldown
    /// - `Err(AppError::NotFound(_))` - Category not found
    /// - `Err(AppError::InternalError(_))` - Discord notification or data fetch failed
    /// - `Err(AppError::Database(_))` - Database operation failed
    pub async fn create(&self, dto: CreateFleetDto, is_admin: bool) -> Result<FleetDto, AppError> {
        let repo = FleetRepository::new(self.db);

        // Parse the fleet time from "YYYY-MM-DD HH:MM" format
        let fleet_time = Self::parse_fleet_time(&dto.fleet_time).map_err(|e| *e)?;

        // Validate fleet time doesn't conflict with existing fleets in the same category
        self.validate_fleet_time_conflict(dto.category_id, fleet_time, None)
            .await?;

        // Create the fleet
        let params = CreateFleetParams {
            category_id: dto.category_id,
            name: dto.name.clone(),
            commander_id: dto.commander_id,
            fleet_time,
            description: dto.description.clone(),
            field_values: dto.field_values.clone(),
            hidden: dto.hidden,
            disable_reminder: dto.disable_reminder,
        };
        let fleet = repo.create(params).await?;

        // Post fleet creation notification to Discord
        let notification_service =
            FleetNotificationService::new(self.db, self.discord_http.clone(), self.app_url.clone());
        notification_service
            .post_fleet_creation(&fleet, &dto.field_values)
            .await?;

        // Fetch the full fleet data with enriched information
        // Get guild_id from the category
        let category = entity::prelude::FleetCategory::find_by_id(dto.category_id)
            .one(self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        let guild_id = category
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Failed to parse guild_id: {}", e)))?;

        self.get_by_id(fleet.id, guild_id, dto.commander_id, is_admin)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet not found after creation".to_string()))
    }

    /// Retrieves a fleet by ID with enriched data and permission filtering.
    ///
    /// Fetches fleet with category name, commander display name, and field values.
    /// Applies visibility rules based on user permissions and fleet hidden status.
    ///
    /// # Arguments
    /// - `id` - Fleet ID to retrieve
    /// - `guild_id` - Discord guild ID for fetching commander nickname and permissions
    /// - `user_id` - Discord user ID for permission checks
    /// - `is_admin` - Whether the user is an admin (bypasses all permission checks)
    ///
    /// # Returns
    /// - `Ok(Some(FleetDto))` - Fleet with enriched data (user has permission to view)
    /// - `Ok(None)` - Fleet not found or user lacks permission to view it
    /// - `Err(AppError::NotFound(_))` - Related entity (category, commander) not found
    /// - `Err(AppError::InternalError(_))` - Failed to parse IDs or fetch guild member
    /// - `Err(AppError::Database(_))` - Database operation failed
    pub async fn get_by_id(
        &self,
        id: i32,
        guild_id: u64,
        user_id: u64,
        is_admin: bool,
    ) -> Result<Option<FleetDto>, AppError> {
        let repo = FleetRepository::new(self.db);
        let category_repo = FleetCategoryRepository::new(self.db);

        let result = repo.get_by_id(id).await?;

        if let Some((fleet, field_values_by_id)) = result {
            // Fetch category
            let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                .one(self.db)
                .await?
                .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

            // Check if user has any permission to view this category (view, create, or manage)
            if !is_admin {
                let can_view = category_repo
                    .user_can_view_category(user_id, guild_id, fleet.category_id)
                    .await?;
                let can_create = category_repo
                    .user_can_create_category(user_id, guild_id, fleet.category_id)
                    .await?;
                let can_manage = category_repo
                    .user_can_manage_category(user_id, guild_id, fleet.category_id)
                    .await?;

                if !can_view && !can_create && !can_manage {
                    // User has no permission to view this category at all
                    return Ok(None);
                }

                // If fleet is hidden, check if user can see it
                if fleet.hidden {
                    // Users with create or manage permission can always see hidden fleets
                    let can_see_hidden = can_create || can_manage;

                    if !can_see_hidden {
                        // User can only see hidden fleet if reminder time has passed or fleet has started
                        let now = chrono::Utc::now();
                        let can_see_by_time = if let Some(reminder_seconds) = category.ping_reminder
                        {
                            // Check if reminder time has passed
                            let reminder_duration =
                                chrono::Duration::seconds(reminder_seconds as i64);
                            let reminder_time = fleet.fleet_time - reminder_duration;
                            now >= reminder_time
                        } else {
                            // No reminder configured, check if fleet has started
                            now >= fleet.fleet_time
                        };

                        if !can_see_by_time {
                            // User cannot see this hidden fleet yet
                            return Ok(None);
                        }
                    }
                }
            }

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

            // Fetch commander nickname from guild
            use crate::server::data::discord::DiscordGuildMemberRepository;
            let member_repo = DiscordGuildMemberRepository::new(self.db);
            let commander_display_name =
                if let Ok(Some(member)) = member_repo.get_member(commander_id, guild_id).await {
                    member.nickname.unwrap_or(member.username)
                } else {
                    commander.name.clone()
                };

            Ok(Some(FleetDto {
                id: fleet.id,
                category_id: fleet.category_id,
                category_name: category.name,
                name: fleet.name,
                commander_id,
                commander_name: commander_display_name,
                fleet_time: fleet.fleet_time,
                description: fleet.description,
                field_values,
                created_at: fleet.created_at,
                hidden: fleet.hidden,
                disable_reminder: fleet.disable_reminder,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves paginated fleets for a guild with permission filtering.
    ///
    /// Returns fleets filtered by category permissions, time (excludes fleets >1 hour old),
    /// and hidden fleet visibility rules. Enriches each fleet with category and commander names.
    ///
    /// # Arguments
    /// - `params` - Guild ID, user ID, admin status, and pagination configuration
    ///
    /// # Returns
    /// - `Ok(PaginatedFleetsDto)` - Paginated list of fleets with total count and page info
    /// - `Err(AppError::InternalError(_))` - Failed to parse IDs or fetch guild member
    /// - `Err(AppError::Database(_))` - Database operation failed
    pub async fn get_paginated_by_guild(
        &self,
        params: GetPaginatedFleetsByGuildParam,
    ) -> Result<PaginatedFleetsDto, AppError> {
        let repo = FleetRepository::new(self.db);
        let category_repo = FleetCategoryRepository::new(self.db);

        // Get viewable category IDs for non-admin users
        let viewable_category_ids = if params.is_admin {
            None // Admins can view all categories
        } else {
            Some(
                category_repo
                    .get_viewable_category_ids_by_user(params.user_id, params.guild_id)
                    .await?,
            )
        };

        // Get categories where user has create or manage permissions (can see hidden fleets)
        let manageable_category_ids = if params.is_admin {
            None // Admins can see all hidden fleets
        } else {
            let create_ids = category_repo
                .get_creatable_category_ids_by_user(params.user_id, params.guild_id)
                .await?;
            let manage_ids = category_repo
                .get_manageable_category_ids_by_user(params.user_id, params.guild_id)
                .await?;

            // Combine create and manage IDs
            let mut combined: std::collections::HashSet<i32> = create_ids.into_iter().collect();
            combined.extend(manage_ids);
            Some(combined.into_iter().collect::<Vec<i32>>())
        };

        let (fleets, total) = repo
            .get_paginated_by_guild(
                params.guild_id,
                params.page,
                params.per_page,
                viewable_category_ids,
            )
            .await?;

        let total_pages = if params.per_page > 0 {
            (total as f64 / params.per_page as f64).ceil() as u64
        } else {
            0
        };

        // Enrich fleet data with category and commander names
        let mut fleet_list = Vec::new();
        let now = chrono::Utc::now();

        for fleet in fleets {
            // Filter hidden fleets based on permissions
            if fleet.hidden {
                // Check if user can see hidden fleets in this category
                let can_see_hidden = params.is_admin
                    || manageable_category_ids
                        .as_ref()
                        .map(|ids| ids.contains(&fleet.category_id))
                        .unwrap_or(false);

                if !can_see_hidden {
                    // User can only see hidden fleet if reminder time has passed
                    // Get the category to check reminder time
                    if let Ok(Some(category)) =
                        entity::prelude::FleetCategory::find_by_id(fleet.category_id)
                            .one(self.db)
                            .await
                    {
                        if let Some(reminder_seconds) = category.ping_reminder {
                            let reminder_time = fleet.fleet_time
                                - chrono::Duration::seconds(reminder_seconds as i64);
                            if now < reminder_time {
                                // Skip this fleet - not yet visible
                                continue;
                            }
                        } else {
                            // No reminder time configured, show at fleet start time
                            if now < fleet.fleet_time {
                                // Skip this fleet - not yet visible
                                continue;
                            }
                        }
                    } else {
                        // If category not found, skip the fleet for safety
                        continue;
                    }
                }
            }
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

                // Fetch commander nickname from guild
                use crate::server::data::discord::DiscordGuildMemberRepository;
                let member_repo = DiscordGuildMemberRepository::new(self.db);
                let commander_display_name = if let Ok(Some(member)) =
                    member_repo.get_member(commander_id, params.guild_id).await
                {
                    member.nickname.unwrap_or(member.username)
                } else {
                    commander.name.clone()
                };

                fleet_list.push(FleetListItemDto {
                    id: fleet.id,
                    category_id: fleet.category_id,
                    category_name: category.name,
                    name: fleet.name,
                    commander_id,
                    commander_name: commander_display_name,
                    fleet_time: fleet.fleet_time,
                    hidden: fleet.hidden,
                    disable_reminder: fleet.disable_reminder,
                });
            }
        }

        Ok(PaginatedFleetsDto {
            fleets: fleet_list,
            total,
            page: params.page,
            per_page: params.per_page,
            total_pages,
        })
    }

    /// Updates a fleet with time validation and Discord notification updates.
    ///
    /// Updates fleet properties after validating time constraints and checking for cooldown
    /// conflicts. For fleets not yet started, validates new time is not too far in past.
    /// For started fleets, ensures new time is not earlier than original. Updates Discord
    /// messages with new information.
    ///
    /// # Arguments
    /// - `id` - Fleet ID to update
    /// - `guild_id` - Discord guild ID for authorization verification
    /// - `user_id` - Discord user ID for fetching result with visibility rules
    /// - `is_admin` - Whether the user is an admin (bypasses visibility rules on result)
    /// - `dto` - Update data including new time, name, description, and field values
    ///
    /// # Returns
    /// - `Ok(FleetDto)` - Updated fleet with enriched data
    /// - `Err(AppError::NotFound(_))` - Fleet, category, or commander not found
    /// - `Err(AppError::BadRequest(_))` - Time validation failed or conflict with cooldown
    /// - `Err(AppError::InternalError(_))` - Discord notification or ID parsing failed
    /// - `Err(AppError::Database(_))` - Database operation failed
    pub async fn update(
        &self,
        id: i32,
        guild_id: u64,
        user_id: u64,
        is_admin: bool,
        dto: UpdateFleetDto,
    ) -> Result<FleetDto, AppError> {
        let repo = FleetRepository::new(self.db);

        // Get the current fleet to verify it belongs to the guild and get original time
        let result = repo.get_by_id(id).await?;
        if let Some((fleet, _)) = result {
            // Parse the fleet time with original time for validation
            let original_time = fleet.fleet_time;
            let new_fleet_time =
                Self::parse_fleet_time_with_min(&dto.fleet_time, Some(original_time))
                    .map_err(|e| *e)?;

            // Validate fleet time doesn't conflict with existing fleets (excluding this fleet)
            self.validate_fleet_time_conflict(dto.category_id, new_fleet_time, Some(id))
                .await?;
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
                let params = UpdateFleetParams {
                    id,
                    category_id: Some(dto.category_id),
                    name: Some(dto.name.clone()),
                    fleet_time: Some(new_fleet_time),
                    description: Some(dto.description.clone()),
                    field_values: Some(dto.field_values.clone()),
                    hidden: Some(dto.hidden),
                    disable_reminder: Some(dto.disable_reminder),
                };
                let updated_fleet = repo.update(params).await?;

                // Update Discord messages with new fleet information
                let notification_service = FleetNotificationService::new(
                    self.db,
                    self.discord_http.clone(),
                    self.app_url.clone(),
                );
                notification_service
                    .update_fleet_messages(&updated_fleet, &dto.field_values)
                    .await?;

                // Fetch the updated fleet data with enriched information
                return self
                    .get_by_id(id, guild_id, user_id, is_admin)
                    .await?
                    .ok_or_else(|| AppError::NotFound("Fleet not found after update".to_string()));
            }
        }

        Err(AppError::NotFound("Fleet not found".to_string()))
    }

    /// Deletes a fleet and cancels its Discord notifications.
    ///
    /// Verifies the fleet belongs to the specified guild before deletion and cancels
    /// all associated Discord messages (creation and reminder notifications).
    ///
    /// # Arguments
    /// - `id` - Fleet ID to delete
    /// - `guild_id` - Discord guild ID for authorization verification
    ///
    /// # Returns
    /// - `Ok(true)` - Fleet was deleted successfully
    /// - `Ok(false)` - Fleet not found or doesn't belong to guild
    /// - `Err(AppError::InternalError(_))` - Discord notification cancellation failed
    /// - `Err(AppError::Database(_))` - Database operation failed
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
                    // Cancel Discord messages before deleting
                    let notification_service = FleetNotificationService::new(
                        self.db,
                        self.discord_http.clone(),
                        self.app_url.clone(),
                    );
                    notification_service.cancel_fleet_messages(&fleet).await?;

                    repo.delete(id).await?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Parses fleet time from string format with validation.
    ///
    /// Accepts "YYYY-MM-DD HH:MM" format or "now" (case-insensitive). Validates the
    /// time is not more than 2 minutes in the past to account for form fill time and
    /// clock skew.
    ///
    /// # Arguments
    /// - `time_str` - Time string in format "YYYY-MM-DD HH:MM" or "now"
    ///
    /// # Returns
    /// - `Ok(DateTime<Utc>)` - Parsed and validated datetime
    /// - `Err(AppError::BadRequest(_))` - Invalid format or time too far in past
    fn parse_fleet_time(time_str: &str) -> Result<DateTime<Utc>, Box<AppError>> {
        Self::parse_fleet_time_with_min(time_str, None)
    }

    /// Parses fleet time with optional minimum time constraint for updates.
    ///
    /// For new fleets or future-scheduled fleets being updated, validates time is not
    /// more than 2 minutes in the past. For fleets already started (min_time in past),
    /// ensures new time is not earlier than the original time.
    ///
    /// # Arguments
    /// - `time_str` - Time string in format "YYYY-MM-DD HH:MM" or "now" (case-insensitive)
    /// - `min_time` - Optional minimum time for updates (original fleet time)
    ///
    /// # Returns
    /// - `Ok(DateTime<Utc>)` - Parsed and validated datetime
    /// - `Err(AppError::BadRequest(_))` - Invalid format or time validation failed
    fn parse_fleet_time_with_min(
        time_str: &str,
        min_time: Option<DateTime<Utc>>,
    ) -> Result<DateTime<Utc>, Box<AppError>> {
        let now = Utc::now();

        // Handle "now" shorthand (case-insensitive)
        let fleet_time = if time_str.trim().eq_ignore_ascii_case("now") {
            now
        } else {
            NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M")
                .map(|naive| naive.and_utc())
                .map_err(|e| {
                    Box::new(AppError::BadRequest(format!(
                        "Invalid fleet time format. Expected 'YYYY-MM-DD HH:MM' or 'now', got '{}': {}",
                        time_str, e
                    )))
                })?
        };

        // If min_time is provided and is in the past, validate against min_time
        if let Some(min_time) = min_time {
            if min_time < now && fleet_time < min_time {
                return Err(Box::new(AppError::BadRequest(format!(
                    "Fleet time cannot be set earlier than the original time ({})",
                    min_time.format("%Y-%m-%d %H:%M UTC")
                ))));
            }
        }

        // Validate fleet time is not in the past (only if min_time is not provided or is in the future)
        // Allow a 2-minute grace period for immediate fleets to handle:
        // - Time spent filling out the form
        // - Clock skew between client and server
        if min_time.is_none() || min_time.map(|t| t >= now).unwrap_or(true) {
            let grace_period = chrono::Duration::minutes(2);
            let min_allowed_time = now - grace_period;

            if fleet_time < min_allowed_time {
                return Err(Box::new(AppError::BadRequest(
                    "Fleet time cannot be more than 2 minutes in the past".to_string(),
                )));
            }
        }

        Ok(fleet_time)
    }

    /// Validates fleet time doesn't conflict with category cooldown settings.
    ///
    /// Checks if the category has a ping_cooldown configured and ensures no other fleet
    /// in the same category is scheduled within the cooldown window (before or after the
    /// proposed time).
    ///
    /// # Arguments
    /// - `category_id` - Category ID to check for conflicts
    /// - `fleet_time` - Proposed fleet time
    /// - `exclude_fleet_id` - Optional fleet ID to exclude from check (for updates)
    ///
    /// # Returns
    /// - `Ok(())` - No conflicts found
    /// - `Err(AppError::NotFound(_))` - Category not found
    /// - `Err(AppError::BadRequest(_))` - Conflict found with existing fleet
    /// - `Err(AppError::Database(_))` - Database operation failed
    async fn validate_fleet_time_conflict(
        &self,
        category_id: i32,
        fleet_time: DateTime<Utc>,
        exclude_fleet_id: Option<i32>,
    ) -> Result<(), AppError> {
        // Get the category to check ping_cooldown setting
        let category = entity::prelude::FleetCategory::find_by_id(category_id)
            .one(self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        // If no ping_cooldown is set, no validation needed
        let Some(cooldown_seconds) = category.ping_cooldown else {
            return Ok(());
        };

        // Calculate the time window to check
        let cooldown_duration = chrono::Duration::seconds(cooldown_seconds as i64);
        let time_window_start = fleet_time - cooldown_duration;
        let time_window_end = fleet_time + cooldown_duration;

        // Query for conflicting fleets in the same category
        let mut query = entity::prelude::Fleet::find()
            .filter(entity::fleet::Column::CategoryId.eq(category_id))
            .filter(entity::fleet::Column::FleetTime.gte(time_window_start))
            .filter(entity::fleet::Column::FleetTime.lte(time_window_end));

        // Exclude the current fleet if updating
        if let Some(exclude_id) = exclude_fleet_id {
            query = query.filter(entity::fleet::Column::Id.ne(exclude_id));
        }

        let conflicting_fleet = query.one(self.db).await?;

        if let Some(conflict) = conflicting_fleet {
            let cooldown_minutes = cooldown_seconds / 60;
            let hours = cooldown_minutes / 60;
            let minutes = cooldown_minutes % 60;

            let cooldown_display = if hours > 0 {
                format!("{} hour(s) {} minute(s)", hours, minutes)
            } else {
                format!("{} minute(s)", minutes)
            };

            return Err(AppError::BadRequest(format!(
                "Fleet time conflicts with another fleet in this category. \
                Category requires a minimum spacing of {} between fleets. \
                Conflicting fleet at {}",
                cooldown_display,
                conflict.fleet_time.format("%Y-%m-%d %H:%M UTC")
            )));
        }

        Ok(())
    }
}
