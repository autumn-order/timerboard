//! Fleet category repository for database operations.
//!
//! This module provides the `FleetCategoryRepository` for managing fleet categories
//! and their associated access controls. It handles CRUD operations, permission checks,
//! and enriched queries that join categories with related entities like ping formats,
//! access roles, ping roles, and channels.
//!
//! All methods return param models at the repository boundary, converting SeaORM
//! entity models internally to prevent database-specific structures from leaking
//! into service and controller layers.

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use std::collections::HashMap;

use crate::server::model::category::{
    CreateFleetCategoryParams, FleetCategoryListItem, FleetCategoryWithCounts,
    FleetCategoryWithRelations, UpdateFleetCategoryParams,
};

/// Repository for fleet category database operations.
///
/// Provides methods for creating, reading, updating, and deleting fleet categories,
/// as well as permission checking and enriched queries with related entities.
pub struct FleetCategoryRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> FleetCategoryRepository<'a> {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    /// - `db` - Database connection for executing queries
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet category with related entities.
    ///
    /// Inserts the category along with its access roles, ping roles, and channels.
    /// This is a transactional operation - if any insert fails, the entire operation
    /// should be rolled back by the database.
    ///
    /// # Arguments
    /// - `params` - Parameters containing category data and related entity IDs
    ///
    /// # Returns
    /// - `Ok(FleetCategoryListItem)` - The created category as a param model
    /// - `Err(DbErr)` - Database error during insertion
    pub async fn create(
        &self,
        params: CreateFleetCategoryParams,
    ) -> Result<FleetCategoryListItem, DbErr> {
        let category = entity::fleet_category::ActiveModel {
            guild_id: ActiveValue::Set(params.guild_id.to_string()),
            ping_format_id: ActiveValue::Set(params.ping_format_id),
            name: ActiveValue::Set(params.name),
            ping_group_id: ActiveValue::Set(params.ping_group_id),
            ping_cooldown: ActiveValue::Set(params.ping_lead_time.map(|d| d.num_seconds() as i32)),
            ping_reminder: ActiveValue::Set(params.ping_reminder.map(|d| d.num_seconds() as i32)),
            max_pre_ping: ActiveValue::Set(params.max_pre_ping.map(|d| d.num_seconds() as i32)),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        // Insert access roles
        for access_role in params.access_roles {
            entity::fleet_category_access_role::ActiveModel {
                fleet_category_id: ActiveValue::Set(category.id),
                role_id: ActiveValue::Set(access_role.role_id.to_string()),
                can_view: ActiveValue::Set(access_role.can_view),
                can_create: ActiveValue::Set(access_role.can_create),
                can_manage: ActiveValue::Set(access_role.can_manage),
            }
            .insert(self.db)
            .await?;
        }

        // Insert ping roles
        for role_id in params.ping_roles {
            entity::fleet_category_ping_role::ActiveModel {
                fleet_category_id: ActiveValue::Set(category.id),
                role_id: ActiveValue::Set(role_id.to_string()),
            }
            .insert(self.db)
            .await?;
        }

        // Insert channels
        for channel_id in params.channels {
            entity::fleet_category_channel::ActiveModel {
                fleet_category_id: ActiveValue::Set(category.id),
                channel_id: ActiveValue::Set(channel_id.to_string()),
            }
            .insert(self.db)
            .await?;
        }

        FleetCategoryListItem::from_entity(category)
    }

    /// Finds a fleet category by ID with all related entities and enriched data.
    ///
    /// Fetches the category along with its ping format, access roles, ping roles,
    /// and channels. Also enriches the roles and channels with display data (name,
    /// color, position) by joining with Discord guild role and channel tables.
    /// Results are sorted by position for consistent display ordering.
    ///
    /// # Arguments
    /// - `id` - Fleet category ID
    ///
    /// # Returns
    /// - `Ok(Some(FleetCategoryWithRelations))` - Category with all related data
    /// - `Ok(None)` - Category not found
    /// - `Err(DbErr)` - Database error during query
    pub async fn find_by_id(&self, id: i32) -> Result<Option<FleetCategoryWithRelations>, DbErr> {
        let category_result = entity::prelude::FleetCategory::find_by_id(id)
            .find_also_related(entity::prelude::PingFormat)
            .one(self.db)
            .await?;

        if let Some((category, ping_format)) = category_result {
            // Fetch access roles
            let access_roles = entity::prelude::FleetCategoryAccessRole::find()
                .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(id))
                .all(self.db)
                .await?;

            // Fetch ping roles
            let ping_roles = entity::prelude::FleetCategoryPingRole::find()
                .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(id))
                .all(self.db)
                .await?;

            // Fetch channels
            let channels = entity::prelude::FleetCategoryChannel::find()
                .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(id))
                .all(self.db)
                .await?;

            // Collect all role IDs
            let mut role_ids: Vec<String> = Vec::new();
            role_ids.extend(access_roles.iter().map(|ar| ar.role_id.clone()));
            role_ids.extend(ping_roles.iter().map(|pr| pr.role_id.clone()));

            // Fetch all roles in one query
            let roles_map: HashMap<String, entity::discord_guild_role::Model> =
                if !role_ids.is_empty() {
                    entity::prelude::DiscordGuildRole::find()
                        .filter(entity::discord_guild_role::Column::RoleId.is_in(role_ids))
                        .filter(entity::discord_guild_role::Column::GuildId.eq(&category.guild_id))
                        .all(self.db)
                        .await?
                        .into_iter()
                        .map(|r| (r.role_id.clone(), r))
                        .collect()
                } else {
                    HashMap::new()
                };

            // Fetch all channels in one query
            let channel_ids: Vec<String> = channels.iter().map(|c| c.channel_id.clone()).collect();
            let channels_map: HashMap<String, entity::discord_guild_channel::Model> =
                if !channel_ids.is_empty() {
                    entity::prelude::DiscordGuildChannel::find()
                        .filter(entity::discord_guild_channel::Column::ChannelId.is_in(channel_ids))
                        .filter(
                            entity::discord_guild_channel::Column::GuildId.eq(&category.guild_id),
                        )
                        .all(self.db)
                        .await?
                        .into_iter()
                        .map(|c| (c.channel_id.clone(), c))
                        .collect()
                } else {
                    HashMap::new()
                };

            // Build enriched results and sort by position
            let mut enriched_access_roles: Vec<(
                entity::fleet_category_access_role::Model,
                Option<entity::discord_guild_role::Model>,
            )> = access_roles
                .into_iter()
                .map(|ar| {
                    let role = roles_map.get(&ar.role_id).cloned();
                    (ar, role)
                })
                .collect();
            // Sort roles by position descending (higher position = higher in Discord UI)
            enriched_access_roles.sort_by(|a, b| {
                let pos_a = a.1.as_ref().map(|r| r.position).unwrap_or(0);
                let pos_b = b.1.as_ref().map(|r| r.position).unwrap_or(0);
                pos_b.cmp(&pos_a)
            });

            let mut enriched_ping_roles: Vec<(
                entity::fleet_category_ping_role::Model,
                Option<entity::discord_guild_role::Model>,
            )> = ping_roles
                .into_iter()
                .map(|pr| {
                    let role = roles_map.get(&pr.role_id).cloned();
                    (pr, role)
                })
                .collect();
            // Sort roles by position descending (higher position = higher in Discord UI)
            enriched_ping_roles.sort_by(|a, b| {
                let pos_a = a.1.as_ref().map(|r| r.position).unwrap_or(0);
                let pos_b = b.1.as_ref().map(|r| r.position).unwrap_or(0);
                pos_b.cmp(&pos_a)
            });

            let mut enriched_channels: Vec<(
                entity::fleet_category_channel::Model,
                Option<entity::discord_guild_channel::Model>,
            )> = channels
                .into_iter()
                .map(|c| {
                    let channel = channels_map.get(&c.channel_id).cloned();
                    (c, channel)
                })
                .collect();
            // Sort channels by position ascending (lower position = higher in Discord UI)
            enriched_channels.sort_by(|a, b| {
                let pos_a = a.1.as_ref().map(|ch| ch.position).unwrap_or(0);
                let pos_b = b.1.as_ref().map(|ch| ch.position).unwrap_or(0);
                pos_a.cmp(&pos_b)
            });

            Ok(Some(FleetCategoryWithRelations {
                category,
                ping_format,
                access_roles: enriched_access_roles,
                ping_roles: enriched_ping_roles,
                channels: enriched_channels,
            }))
        } else {
            Ok(None)
        }
    }

    /// Gets paginated fleet categories for a guild with counts of related entities.
    ///
    /// Retrieves categories ordered by name with related ping formats and counts of
    /// access roles, ping roles, and channels. This is more efficient than fetching
    /// full related data when only counts are needed for list views.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID
    /// - `page` - Page number (0-indexed)
    /// - `per_page` - Number of items per page
    ///
    /// # Returns
    /// - `Ok((categories, total))` - Tuple of category list and total count
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_by_guild_id_paginated(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<FleetCategoryWithCounts>, u64), DbErr> {
        let paginator = entity::prelude::FleetCategory::find()
            .find_also_related(entity::prelude::PingFormat)
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::fleet_category::Column::Name)
            .paginate(self.db, per_page);

        let total = paginator.num_items().await?;
        let categories = paginator.fetch_page(page).await?;

        // Fetch counts for each category
        let mut results = Vec::new();
        for (category, ping_format) in categories {
            let access_roles_count = entity::prelude::FleetCategoryAccessRole::find()
                .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category.id))
                .count(self.db)
                .await? as usize;

            let ping_roles_count = entity::prelude::FleetCategoryPingRole::find()
                .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category.id))
                .count(self.db)
                .await? as usize;

            let channels_count = entity::prelude::FleetCategoryChannel::find()
                .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(category.id))
                .count(self.db)
                .await? as usize;

            results.push(FleetCategoryWithCounts {
                category,
                ping_format,
                access_roles_count,
                ping_roles_count,
                channels_count,
            });
        }

        Ok((results, total))
    }

    /// Updates a fleet category and replaces all related entities.
    ///
    /// Updates the category's core fields (name, ping format, durations) and completely
    /// replaces all access roles, ping roles, and channels with the new data provided.
    /// Existing related entities are deleted before inserting new ones.
    ///
    /// # Arguments
    /// - `params` - Parameters containing updated category data and related entity IDs
    ///
    /// # Returns
    /// - `Ok(FleetCategoryListItem)` - The updated category as a param model
    /// - `Err(DbErr::RecordNotFound)` - Category with specified ID not found
    /// - `Err(DbErr)` - Database error during update or related entity operations
    pub async fn update(
        &self,
        params: UpdateFleetCategoryParams,
    ) -> Result<FleetCategoryListItem, DbErr> {
        let category = entity::prelude::FleetCategory::find_by_id(params.id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Fleet category with id {} not found",
                params.id
            )))?;

        let mut active_model: entity::fleet_category::ActiveModel = category.into();
        active_model.ping_format_id = ActiveValue::Set(params.ping_format_id);
        active_model.name = ActiveValue::Set(params.name);
        active_model.ping_group_id = ActiveValue::Set(params.ping_group_id);
        active_model.ping_cooldown =
            ActiveValue::Set(params.ping_lead_time.map(|d| d.num_seconds() as i32));
        active_model.ping_reminder =
            ActiveValue::Set(params.ping_reminder.map(|d| d.num_seconds() as i32));
        active_model.max_pre_ping =
            ActiveValue::Set(params.max_pre_ping.map(|d| d.num_seconds() as i32));

        let updated_category = active_model.update(self.db).await?;

        // Delete existing related entities
        entity::prelude::FleetCategoryAccessRole::delete_many()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(params.id))
            .exec(self.db)
            .await?;

        entity::prelude::FleetCategoryPingRole::delete_many()
            .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(params.id))
            .exec(self.db)
            .await?;

        entity::prelude::FleetCategoryChannel::delete_many()
            .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(params.id))
            .exec(self.db)
            .await?;

        // Insert new access roles
        for access_role in params.access_roles {
            entity::fleet_category_access_role::ActiveModel {
                fleet_category_id: ActiveValue::Set(params.id),
                role_id: ActiveValue::Set(access_role.role_id.to_string()),
                can_view: ActiveValue::Set(access_role.can_view),
                can_create: ActiveValue::Set(access_role.can_create),
                can_manage: ActiveValue::Set(access_role.can_manage),
            }
            .insert(self.db)
            .await?;
        }

        // Insert new ping roles
        for role_id in params.ping_roles {
            entity::fleet_category_ping_role::ActiveModel {
                fleet_category_id: ActiveValue::Set(params.id),
                role_id: ActiveValue::Set(role_id.to_string()),
            }
            .insert(self.db)
            .await?;
        }

        // Insert new channels
        for channel_id in params.channels {
            entity::fleet_category_channel::ActiveModel {
                fleet_category_id: ActiveValue::Set(params.id),
                channel_id: ActiveValue::Set(channel_id.to_string()),
            }
            .insert(self.db)
            .await?;
        }

        FleetCategoryListItem::from_entity(updated_category)
    }

    /// Deletes a fleet category and all related entities.
    ///
    /// Deletes the category by ID. Related entities (access roles, ping roles, channels)
    /// are automatically deleted via database cascade constraints.
    ///
    /// # Arguments
    /// - `id` - Fleet category ID to delete
    ///
    /// # Returns
    /// - `Ok(())` - Category successfully deleted
    /// - `Err(DbErr)` - Database error during deletion
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::FleetCategory::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// Checks if a fleet category exists and belongs to the specified guild.
    ///
    /// Used for validation before performing operations that require guild ownership.
    ///
    /// # Arguments
    /// - `id` - Fleet category ID to check
    /// - `guild_id` - Discord guild ID that should own the category
    ///
    /// # Returns
    /// - `Ok(true)` - Category exists and belongs to the guild
    /// - `Ok(false)` - Category not found or belongs to a different guild
    /// - `Err(DbErr)` - Database error during query
    pub async fn exists_in_guild(&self, id: i32, guild_id: u64) -> Result<bool, DbErr> {
        let count = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::Id.eq(id))
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .count(self.db)
            .await?;

        Ok(count > 0)
    }

    /// Gets fleet categories that use a specific ping format.
    ///
    /// Used to check dependencies before deleting a ping format or to find
    /// categories that need updating when a ping format changes.
    ///
    /// # Arguments
    /// - `ping_format_id` - Ping format ID to search for
    ///
    /// # Returns
    /// - `Ok(Vec<FleetCategoryListItem>)` - Categories using the specified ping format
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<FleetCategoryListItem>, DbErr> {
        let categories = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::PingFormatId.eq(ping_format_id))
            .all(self.db)
            .await?;

        categories
            .into_iter()
            .map(FleetCategoryListItem::from_entity)
            .collect()
    }

    /// Gets category IDs that post to a specific channel.
    ///
    /// Retrieves all category IDs that are configured to post fleet notifications
    /// to the specified Discord channel. Used for building the upcoming fleets list
    /// which aggregates fleets from all categories posting to a channel.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID to search for
    ///
    /// # Returns
    /// - `Ok(Vec<i32>)` - Category IDs that post to this channel
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_category_ids_by_channel(&self, channel_id: u64) -> Result<Vec<i32>, DbErr> {
        let category_channels = entity::prelude::FleetCategoryChannel::find()
            .filter(entity::fleet_category_channel::Column::ChannelId.eq(channel_id.to_string()))
            .all(self.db)
            .await?;

        Ok(category_channels
            .into_iter()
            .map(|cc| cc.fleet_category_id)
            .collect())
    }

    /// Gets category data by ID as a simple map for lookups.
    ///
    /// Retrieves multiple categories and returns them as a HashMap keyed by ID,
    /// useful for looking up category names when building fleet lists.
    ///
    /// # Arguments
    /// - `category_ids` - List of category IDs to retrieve
    ///
    /// # Returns
    /// - `Ok(HashMap<i32, String>)` - Map of category ID to category name
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_names_by_ids(
        &self,
        category_ids: Vec<i32>,
    ) -> Result<HashMap<i32, String>, DbErr> {
        let categories = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?;

        Ok(categories.into_iter().map(|c| (c.id, c.name)).collect())
    }
}
