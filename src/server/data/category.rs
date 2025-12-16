use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use std::collections::HashMap;

use crate::server::model::category::{
    CreateFleetCategoryParams, FleetCategoryWithCounts, FleetCategoryWithRelations,
    UpdateFleetCategoryParams,
};

pub struct FleetCategoryRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetCategoryRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet category and returns it with related ping format
    pub async fn create(
        &self,
        params: CreateFleetCategoryParams,
    ) -> Result<entity::fleet_category::Model, DbErr> {
        let category = entity::fleet_category::ActiveModel {
            guild_id: ActiveValue::Set(params.guild_id.to_string()),
            ping_format_id: ActiveValue::Set(params.ping_format_id),
            name: ActiveValue::Set(params.name),
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

        Ok(category)
    }

    /// Gets a fleet category by ID with related ping format and all related entities with enriched data
    pub async fn get_by_id(&self, id: i32) -> Result<Option<FleetCategoryWithRelations>, DbErr> {
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

    /// Gets paginated fleet categories for a guild with related ping format and counts
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

    /// Updates a fleet category's name and duration fields and returns it with related ping format
    pub async fn update(
        &self,
        params: UpdateFleetCategoryParams,
    ) -> Result<entity::fleet_category::Model, DbErr> {
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

        Ok(updated_category)
    }

    /// Deletes a fleet category
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::FleetCategory::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// Checks if a fleet category exists and belongs to the specified guild
    pub async fn exists_in_guild(&self, id: i32, guild_id: u64) -> Result<bool, DbErr> {
        let count = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::Id.eq(id))
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .count(self.db)
            .await?;

        Ok(count > 0)
    }

    /// Gets fleet categories by ping format ID
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<entity::fleet_category::Model>, DbErr> {
        entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::PingFormatId.eq(ping_format_id))
            .all(self.db)
            .await
    }

    /// Gets fleet categories that a user can create or manage
    ///
    /// Returns categories where the user has can_create OR can_manage permission
    /// through their Discord roles. Admins are not handled here - check admin status
    /// before calling this method.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_id`: Discord guild ID (u64)
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of fleet categories the user can create/manage
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_manageable_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<entity::fleet_category::Model>, DbErr> {
        use sea_orm::Condition;

        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_create or can_manage permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(
                Condition::any()
                    .add(entity::fleet_category_access_role::Column::CanCreate.eq(true))
                    .add(entity::fleet_category_access_role::Column::CanManage.eq(true)),
            )
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Get the actual category models for this guild
        entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .order_by_asc(entity::fleet_category::Column::Name)
            .all(self.db)
            .await
    }

    /// Gets fleet category IDs that a user can view
    ///
    /// Returns category IDs where the user has can_view permission
    /// through their Discord roles. Admins are not handled here - check admin status
    /// before calling this method.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_id`: Discord guild ID (u64)
    ///
    /// # Returns
    /// - `Ok(Vec<i32>)`: Vector of fleet category IDs the user can view
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_viewable_category_ids_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<i32>, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_view permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanView.eq(true))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Verify these categories belong to the specified guild
        let guild_category_ids: Vec<i32> = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        Ok(guild_category_ids)
    }

    /// Gets category IDs where user has create permission
    pub async fn get_creatable_category_ids_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<i32>, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_create permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanCreate.eq(true))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Verify these categories belong to the specified guild
        let guild_category_ids: Vec<i32> = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        Ok(guild_category_ids)
    }

    /// Gets category IDs where user has manage permission
    pub async fn get_manageable_category_ids_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<i32>, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_manage permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanManage.eq(true))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Verify these categories belong to the specified guild
        let guild_category_ids: Vec<i32> = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        Ok(guild_category_ids)
    }

    /// Checks if a user has view access to a specific category
    pub async fn user_can_view_category(
        &self,
        user_id: u64,
        _guild_id: u64,
        category_id: i32,
    ) -> Result<bool, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(false);
        }

        // Check if any of the user's roles have view access to this category
        let access_count = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category_id))
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanView.eq(true))
            .count(self.db)
            .await?;

        Ok(access_count > 0)
    }

    /// Checks if a user has create access to a specific category
    pub async fn user_can_create_category(
        &self,
        user_id: u64,
        _guild_id: u64,
        category_id: i32,
    ) -> Result<bool, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(false);
        }

        // Check if any of the user's roles have create access to this category
        let access_count = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category_id))
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanCreate.eq(true))
            .count(self.db)
            .await?;

        Ok(access_count > 0)
    }

    /// Checks if a user has manage access to a specific category
    pub async fn user_can_manage_category(
        &self,
        user_id: u64,
        _guild_id: u64,
        category_id: i32,
    ) -> Result<bool, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(false);
        }

        // Check if any of the user's roles have manage access to this category
        let access_count = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category_id))
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanManage.eq(true))
            .count(self.db)
            .await?;

        Ok(access_count > 0)
    }

    /// Gets category details with ping format fields for fleet creation
    pub async fn get_category_details(
        &self,
        category_id: i32,
    ) -> Result<Option<FleetCategoryWithRelations>, DbErr> {
        self.get_by_id(category_id).await
    }
}
