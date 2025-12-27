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
    model::fleet::{FleetDto, FleetListItemDto, PaginatedFleetsDto, UpdateFleetDto},
    server::{
        data::{
            category::FleetCategoryRepository, discord::DiscordGuildMemberRepository,
            fleet::FleetRepository, ping_format::field::PingFormatFieldRepository,
            ping_group::PingGroupRepository, user::UserRepository,
            user_category_permission::UserCategoryPermissionRepository,
        },
        error::AppError,
        model::fleet::{CreateFleetParam, GetPaginatedFleetsByGuildParam, UpdateFleetParam},
        service::fleet_notification::FleetNotificationService,
        util::parse::parse_u64_from_string,
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
    pub async fn create(
        &self,
        param: CreateFleetParam,
        is_admin: bool,
    ) -> Result<FleetDto, AppError> {
        let fleet_repo = FleetRepository::new(self.db);

        // Validate fleet time doesn't conflict with existing fleets in the same category
        self.validate_fleet_time_conflict(param.category_id, param.fleet_time, None)
            .await?;

        let field_values = param.field_values.clone();
        let fleet = fleet_repo.create(param).await?;

        // Post fleet creation notification to Discord
        let notification_service =
            FleetNotificationService::new(self.db, self.discord_http.clone(), self.app_url.clone());
        notification_service
            .post_fleet_creation(&fleet, &field_values)
            .await?;

        // Update upcoming fleets lists for all channels in this category
        self.update_upcoming_fleets_lists_for_category(fleet.category_id)
            .await?;

        // Fetch the full fleet data with enriched information
        // Get guild_id from the category

        self.get_by_id(fleet.id, fleet.commander_id, is_admin)
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
    /// - `Err(AppError::Database(_))` - Database operation failed
    pub async fn get_by_id(
        &self,
        id: i32,
        user_id: u64,
        is_admin: bool,
    ) -> Result<Option<FleetDto>, AppError> {
        let user_repo = UserRepository::new(self.db);
        let category_repo = FleetCategoryRepository::new(self.db);
        let ping_format_field_repo = PingFormatFieldRepository::new(self.db);
        let fleet_repo = FleetRepository::new(self.db);
        let member_repo = DiscordGuildMemberRepository::new(self.db);

        let result = fleet_repo.get_by_id(id).await?;

        if let Some((fleet, field_values_by_id)) = result {
            let Some(category) = category_repo
                .find_by_id(fleet.category_id)
                .await?
                .map(|c| c.category)
            else {
                return Err(AppError::NotFound("Category not found".to_string()));
            };

            // Check if user has any permission to view this category (view, create, or manage)
            if !is_admin {
                let permission_repo = UserCategoryPermissionRepository::new(self.db);
                let can_view = permission_repo
                    .user_can_view_category(user_id, fleet.category_id)
                    .await?;
                let can_create = permission_repo
                    .user_can_create_category(user_id, fleet.category_id)
                    .await?;
                let can_manage = permission_repo
                    .user_can_manage_category(user_id, fleet.category_id)
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
            let Some(commander) = user_repo.find_by_id(fleet.commander_id).await? else {
                return Err(AppError::NotFound("Fleet commander not found".to_string()));
            };

            // Fetch field names for the ping format
            let guild_id = parse_u64_from_string(category.guild_id.clone())?;
            let fields = ping_format_field_repo
                .get_by_ping_format_id(guild_id, category.ping_format_id)
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

            let category_guild_id = parse_u64_from_string(category.guild_id)?;

            // Fetch commander nickname from guild
            let commander_display_name = if let Ok(Some(member)) = member_repo
                .get_member(commander.discord_id, category_guild_id)
                .await
            {
                member.nickname.unwrap_or(member.username)
            } else {
                commander.name.clone()
            };

            Ok(Some(FleetDto {
                id: fleet.id,
                category_id: fleet.category_id,
                category_name: category.name,
                name: fleet.name,
                commander_id: commander.discord_id,
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
        let user_repo = UserRepository::new(self.db);
        let category_repo = FleetCategoryRepository::new(self.db);
        let fleet_repo = FleetRepository::new(self.db);
        let permission_repo = UserCategoryPermissionRepository::new(self.db);

        // Get viewable category IDs for non-admin users
        let viewable_category_ids = if params.is_admin {
            None // Admins can view all categories
        } else {
            Some(
                permission_repo
                    .get_viewable_category_ids_by_user(params.user_id, params.guild_id)
                    .await?,
            )
        };

        // Get categories where user has create or manage permissions (can see hidden fleets)
        let manageable_category_ids = if params.is_admin {
            None // Admins can see all hidden fleets
        } else {
            let create_ids = permission_repo
                .get_creatable_category_ids_by_user(params.user_id, params.guild_id)
                .await?;
            let manage_ids = permission_repo
                .get_manageable_category_ids_by_user(params.user_id, params.guild_id)
                .await?;

            // Combine create and manage IDs
            let mut combined: std::collections::HashSet<i32> = create_ids.into_iter().collect();
            combined.extend(manage_ids);
            Some(combined.into_iter().collect::<Vec<i32>>())
        };

        let (fleets, total) = fleet_repo
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
            let category = category_repo.find_by_id(fleet.category_id).await?;

            // Fetch commander
            let commander = user_repo.find_by_id(fleet.commander_id).await?;

            if let (Some(category), Some(commander)) = (category, commander) {
                // Fetch commander nickname from guild
                let member_repo = DiscordGuildMemberRepository::new(self.db);
                let commander_display_name = if let Ok(Some(member)) = member_repo
                    .get_member(commander.discord_id, params.guild_id)
                    .await
                {
                    member.nickname.unwrap_or(member.username)
                } else {
                    commander.name.clone()
                };

                fleet_list.push(FleetListItemDto {
                    id: fleet.id,
                    category_id: fleet.category_id,
                    category_name: category.category.name,
                    name: fleet.name,
                    commander_id: commander.discord_id,
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
        let category_repo = FleetCategoryRepository::new(self.db);
        let fleet_repo = FleetRepository::new(self.db);

        // Get the current fleet to verify it belongs to the guild and get original time
        let result = fleet_repo.get_by_id(id).await?;
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
            let old_category = category_repo.find_by_id(fleet.category_id).await?;

            if let Some(old_category) = old_category {
                let category_guild_id = parse_u64_from_string(old_category.category.guild_id)?;

                if category_guild_id != guild_id {
                    return Err(AppError::NotFound("Fleet not found".to_string()));
                }

                // If category is being changed, validate the new category belongs to the same guild
                if dto.category_id != fleet.category_id {
                    let Some(new_category) = category_repo.find_by_id(dto.category_id).await?
                    else {
                        return Err(AppError::NotFound("New category not found".to_string()));
                    };

                    let new_category_guild_id =
                        parse_u64_from_string(new_category.category.guild_id)?;

                    if new_category_guild_id != guild_id {
                        return Err(AppError::BadRequest(
                            "New category does not belong to this guild".to_string(),
                        ));
                    }
                }

                // Update the fleet
                let params = UpdateFleetParam {
                    id,
                    category_id: Some(dto.category_id),
                    name: Some(dto.name.clone()),
                    fleet_time: Some(new_fleet_time),
                    description: Some(dto.description.clone()),
                    field_values: Some(dto.field_values.clone()),
                    hidden: Some(dto.hidden),
                    disable_reminder: Some(dto.disable_reminder),
                };
                let updated_fleet = fleet_repo.update(params).await?;

                // Update Discord messages with new fleet information
                let notification_service = FleetNotificationService::new(
                    self.db,
                    self.discord_http.clone(),
                    self.app_url.clone(),
                );
                notification_service
                    .update_fleet_messages(&updated_fleet, &dto.field_values)
                    .await?;

                // Update upcoming fleets lists for all channels in this category
                self.update_upcoming_fleets_lists_for_category(dto.category_id)
                    .await?;

                // If category was changed, also update the old category's channels
                if dto.category_id != fleet.category_id {
                    self.update_upcoming_fleets_lists_for_category(fleet.category_id)
                        .await?;
                }

                // Fetch the updated fleet data with enriched information
                return self
                    .get_by_id(id, user_id, is_admin)
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
        let category_repo = FleetCategoryRepository::new(self.db);
        let fleet_repo = FleetRepository::new(self.db);

        // Check if fleet exists and belongs to the guild
        let result = fleet_repo.get_by_id(id).await?;

        if let Some((fleet, _)) = result {
            // Fetch category to verify guild
            let category = category_repo.find_by_id(fleet.category_id).await?;

            if let Some(category) = category {
                let category_guild_id = parse_u64_from_string(category.category.guild_id)?;

                if category_guild_id == guild_id {
                    // Cancel Discord messages before deleting
                    let notification_service = FleetNotificationService::new(
                        self.db,
                        self.discord_http.clone(),
                        self.app_url.clone(),
                    );
                    notification_service
                        .cancel_fleet_messages(&fleet, self.app_url.as_str())
                        .await?;

                    fleet_repo.delete(id).await?;

                    // Update upcoming fleets lists for all channels in this category
                    self.update_upcoming_fleets_lists_for_category(fleet.category_id)
                        .await?;

                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Updates upcoming fleets lists for all channels configured for a category.
    ///
    /// Called after fleet updates or deletions to reflect changes in the upcoming
    /// fleets lists without bumping them to the most recent message.
    ///
    /// # Arguments
    /// - `category_id` - Category ID whose channels should be updated
    ///
    /// # Returns
    /// - `Ok(())` - Successfully updated all channel lists
    /// - `Err(AppError)` - Database or Discord error
    async fn update_upcoming_fleets_lists_for_category(
        &self,
        category_id: i32,
    ) -> Result<(), AppError> {
        let category_repo = FleetCategoryRepository::new(self.db);

        // Get the category to extract its channels
        let Some(category_data) = category_repo.find_by_id(category_id).await? else {
            return Ok(()); // Category not found, nothing to update
        };

        // Extract channel IDs from the category's channels
        let channel_ids: Vec<u64> = category_data
            .channels
            .iter()
            .filter_map(|(channel_model, _)| channel_model.channel_id.parse::<u64>().ok())
            .collect();

        if channel_ids.is_empty() {
            return Ok(()); // No channels configured
        }

        // Update the upcoming fleets list for each channel
        let notification_service =
            FleetNotificationService::new(self.db, self.discord_http.clone(), self.app_url.clone());

        for channel_id in channel_ids {
            if let Err(e) = notification_service
                .update_upcoming_fleets_list(channel_id)
                .await
            {
                dioxus_logger::tracing::warn!(
                    "Failed to update upcoming fleets list for channel {}: {}",
                    channel_id,
                    e
                );
                // Continue updating other channels even if one fails
            }
        }

        Ok(())
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
        let fleet_category_repo = FleetCategoryRepository::new(self.db);

        // Get the category to check ping_cooldown and ping_group settings
        let Some(category) = fleet_category_repo.find_by_id(category_id).await? else {
            return Err(AppError::NotFound("Category not found".to_string()));
        };

        let guild_id = parse_u64_from_string(category.category.guild_id.clone())?;

        // Check ping group cooldown first (shared across all categories in the group)
        if let Some(ping_group_id) = category.category.ping_group_id {
            let ping_group_repo = PingGroupRepository::new(self.db);
            if let Some(ping_group) = ping_group_repo.find_by_id(guild_id, ping_group_id).await? {
                if let Some(group_cooldown) = ping_group.cooldown {
                    let cooldown_seconds = group_cooldown.num_seconds() as i32;
                    let cooldown_duration = chrono::Duration::seconds(cooldown_seconds as i64);
                    let time_window_start = fleet_time - cooldown_duration;
                    let time_window_end = fleet_time + cooldown_duration;

                    // Get all categories in the same ping group
                    let all_categories = entity::prelude::FleetCategory::find()
                        .filter(entity::fleet_category::Column::PingGroupId.eq(ping_group_id))
                        .all(self.db)
                        .await?;

                    let category_ids: Vec<i32> = all_categories.iter().map(|c| c.id).collect();

                    // Query for conflicting fleets across ALL categories in the ping group
                    let mut query = entity::prelude::Fleet::find()
                        .filter(entity::fleet::Column::CategoryId.is_in(category_ids))
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
                            "Fleet time conflicts with another fleet in ping group '{}'. \
                            This group has a shared cooldown of {} between all fleets. \
                            Conflicting fleet at {}",
                            ping_group.name,
                            cooldown_display,
                            conflict.fleet_time.format("%Y-%m-%d %H:%M UTC")
                        )));
                    }
                }
            }
        }

        // Also check category-specific cooldown (applies only to fleets in this category)
        if let Some(cooldown_seconds) = category.category.ping_cooldown {
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
        }

        Ok(())
    }
}
