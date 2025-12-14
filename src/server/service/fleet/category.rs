use chrono::Duration;
use sea_orm::DatabaseConnection;

use crate::{
    model::fleet::{
        FleetCategoryAccessRoleDto, FleetCategoryChannelDto, FleetCategoryDto,
        FleetCategoryListItemDto, FleetCategoryPingRoleDto, PaginatedFleetCategoriesDto,
    },
    server::{data::fleet::FleetCategoryRepository, error::AppError},
};

pub struct FleetCategoryService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetCategoryService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet category for a guild
    pub async fn create(
        &self,
        guild_id: i64,
        ping_format_id: i32,
        name: String,
        ping_lead_time: Option<Duration>,
        ping_reminder: Option<Duration>,
        max_pre_ping: Option<Duration>,
        access_roles: Vec<FleetCategoryAccessRoleDto>,
        ping_roles: Vec<FleetCategoryPingRoleDto>,
        channels: Vec<FleetCategoryChannelDto>,
    ) -> Result<FleetCategoryDto, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Convert DTOs to repository format
        let access_roles_data: Vec<(i64, bool, bool, bool)> = access_roles
            .iter()
            .map(|ar| (ar.role_id, ar.can_view, ar.can_create, ar.can_manage))
            .collect();

        let ping_roles_data: Vec<i64> = ping_roles.iter().map(|pr| pr.role_id).collect();

        let channels_data: Vec<i64> = channels.iter().map(|c| c.channel_id).collect();

        let (category, ping_format) = repo
            .create(
                guild_id,
                ping_format_id,
                name,
                ping_lead_time,
                ping_reminder,
                max_pre_ping,
                access_roles_data,
                ping_roles_data,
                channels_data,
            )
            .await?;

        Ok(FleetCategoryDto {
            id: category.id,
            guild_id: category.guild_id,
            ping_format_id: category.ping_format_id,
            ping_format_name: ping_format
                .map(|pf| pf.name)
                .unwrap_or_else(|| "Unknown".to_string()),
            name: category.name,
            ping_lead_time: category.ping_cooldown.map(|s| Duration::seconds(s as i64)),
            ping_reminder: category.ping_reminder.map(|s| Duration::seconds(s as i64)),
            max_pre_ping: category.max_pre_ping.map(|s| Duration::seconds(s as i64)),
            access_roles,
            ping_roles,
            channels,
        })
    }

    /// Gets a specific fleet category by ID with all related data
    pub async fn get_by_id(&self, id: i32) -> Result<Option<FleetCategoryDto>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let result = repo.get_by_id(id).await?;

        if let Some((
            category,
            ping_format,
            access_roles_with_details,
            ping_roles_with_details,
            channels_with_details,
        )) = result
        {
            let access_roles = access_roles_with_details
                .into_iter()
                .map(|(ar, role_model)| FleetCategoryAccessRoleDto {
                    role_id: ar.role_id,
                    role_name: role_model
                        .as_ref()
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| format!("Unknown Role ({})", ar.role_id)),
                    role_color: role_model
                        .as_ref()
                        .map(|r| r.color.clone())
                        .unwrap_or_else(|| "#99aab5".to_string()),
                    can_view: ar.can_view,
                    can_create: ar.can_create,
                    can_manage: ar.can_manage,
                })
                .collect();

            let ping_roles = ping_roles_with_details
                .into_iter()
                .map(|(pr, role_model)| FleetCategoryPingRoleDto {
                    role_id: pr.role_id,
                    role_name: role_model
                        .as_ref()
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| format!("Unknown Role ({})", pr.role_id)),
                    role_color: role_model
                        .as_ref()
                        .map(|r| r.color.clone())
                        .unwrap_or_else(|| "#99aab5".to_string()),
                })
                .collect();

            let channels = channels_with_details
                .into_iter()
                .map(|(c, channel_model)| FleetCategoryChannelDto {
                    channel_id: c.channel_id,
                    channel_name: channel_model
                        .as_ref()
                        .map(|ch| ch.name.clone())
                        .unwrap_or_else(|| format!("Unknown Channel ({})", c.channel_id)),
                })
                .collect();

            Ok(Some(FleetCategoryDto {
                id: category.id,
                guild_id: category.guild_id,
                ping_format_id: category.ping_format_id,
                ping_format_name: ping_format
                    .map(|pf| pf.name)
                    .unwrap_or_else(|| "Unknown".to_string()),
                name: category.name,
                ping_lead_time: category.ping_cooldown.map(|s| Duration::seconds(s as i64)),
                ping_reminder: category.ping_reminder.map(|s| Duration::seconds(s as i64)),
                max_pre_ping: category.max_pre_ping.map(|s| Duration::seconds(s as i64)),
                access_roles,
                ping_roles,
                channels,
            }))
        } else {
            Ok(None)
        }
    }

    /// Gets paginated fleet categories for a guild with counts
    pub async fn get_paginated(
        &self,
        guild_id: i64,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedFleetCategoriesDto, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let (categories, total) = repo
            .get_by_guild_id_paginated(guild_id, page, per_page)
            .await?;

        let total_pages = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as u64
        } else {
            0
        };

        Ok(PaginatedFleetCategoriesDto {
            categories: categories
                .into_iter()
                .map(
                    |(c, ping_format, access_roles_count, ping_roles_count, channels_count)| {
                        FleetCategoryListItemDto {
                            id: c.id,
                            guild_id: c.guild_id,
                            ping_format_id: c.ping_format_id,
                            ping_format_name: ping_format
                                .map(|pf| pf.name)
                                .unwrap_or_else(|| "Unknown".to_string()),
                            name: c.name,
                            ping_lead_time: c.ping_cooldown.map(|s| Duration::seconds(s as i64)),
                            ping_reminder: c.ping_reminder.map(|s| Duration::seconds(s as i64)),
                            max_pre_ping: c.max_pre_ping.map(|s| Duration::seconds(s as i64)),
                            access_roles_count,
                            ping_roles_count,
                            channels_count,
                        }
                    },
                )
                .collect(),
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Updates a fleet category's name and duration fields
    /// Returns None if the category doesn't exist or doesn't belong to the guild
    pub async fn update(
        &self,
        id: i32,
        guild_id: i64,
        ping_format_id: i32,
        name: String,
        ping_lead_time: Option<Duration>,
        ping_reminder: Option<Duration>,
        max_pre_ping: Option<Duration>,
        access_roles: Vec<FleetCategoryAccessRoleDto>,
        ping_roles: Vec<FleetCategoryPingRoleDto>,
        channels: Vec<FleetCategoryChannelDto>,
    ) -> Result<Option<FleetCategoryDto>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Check if category exists and belongs to the guild
        if !repo.exists_in_guild(id, guild_id).await? {
            return Ok(None);
        }

        // Convert DTOs to repository format
        let access_roles_data: Vec<(i64, bool, bool, bool)> = access_roles
            .iter()
            .map(|ar| (ar.role_id, ar.can_view, ar.can_create, ar.can_manage))
            .collect();

        let ping_roles_data: Vec<i64> = ping_roles.iter().map(|pr| pr.role_id).collect();

        let channels_data: Vec<i64> = channels.iter().map(|c| c.channel_id).collect();

        let (category, ping_format) = repo
            .update(
                id,
                ping_format_id,
                name,
                ping_lead_time,
                ping_reminder,
                max_pre_ping,
                access_roles_data,
                ping_roles_data,
                channels_data,
            )
            .await?;

        Ok(Some(FleetCategoryDto {
            id: category.id,
            guild_id: category.guild_id,
            ping_format_id: category.ping_format_id,
            ping_format_name: ping_format
                .map(|pf| pf.name)
                .unwrap_or_else(|| "Unknown".to_string()),
            name: category.name,
            ping_lead_time: category.ping_cooldown.map(|s| Duration::seconds(s as i64)),
            ping_reminder: category.ping_reminder.map(|s| Duration::seconds(s as i64)),
            max_pre_ping: category.max_pre_ping.map(|s| Duration::seconds(s as i64)),
            access_roles,
            ping_roles,
            channels,
        }))
    }

    /// Deletes a fleet category
    /// Returns true if deleted, false if not found or doesn't belong to guild
    pub async fn delete(&self, id: i32, guild_id: i64) -> Result<bool, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Check if category exists and belongs to the guild
        if !repo.exists_in_guild(id, guild_id).await? {
            return Ok(false);
        }

        repo.delete(id).await?;

        Ok(true)
    }

    /// Gets fleet categories by ping format ID
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<FleetCategoryListItemDto>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let categories = repo.get_by_ping_format_id(ping_format_id).await?;

        Ok(categories
            .into_iter()
            .map(|c| FleetCategoryListItemDto {
                id: c.id,
                guild_id: c.guild_id,
                ping_format_id: c.ping_format_id,
                ping_format_name: String::new(), // Not needed for this use case
                name: c.name,
                ping_lead_time: c.ping_cooldown.map(|s| Duration::seconds(s as i64)),
                ping_reminder: c.ping_reminder.map(|s| Duration::seconds(s as i64)),
                max_pre_ping: c.max_pre_ping.map(|s| Duration::seconds(s as i64)),
                access_roles_count: 0,
                ping_roles_count: 0,
                channels_count: 0,
            })
            .collect())
    }
}
