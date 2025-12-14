use dioxus_logger::tracing;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use serenity::all::{Role, RoleId};
use std::collections::HashMap;

use crate::{
    model::discord::{DiscordGuildRoleDto, PaginatedDiscordGuildRolesDto},
    server::{data::discord::DiscordGuildRoleRepository, error::AppError},
};

pub struct DiscordGuildRoleService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRoleService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Updates roles for a guild by deleting roles that no longer exist and upserting current roles
    pub async fn update_roles(
        &self,
        guild_id: u64,
        guild_roles: &HashMap<RoleId, Role>,
    ) -> Result<(), AppError> {
        let role_repo = DiscordGuildRoleRepository::new(self.db);

        // Get existing roles from database
        let existing_roles = role_repo.get_by_guild_id(guild_id).await?;

        // Find roles that no longer exist in Discord and delete them
        for existing_role in existing_roles {
            let role_id = existing_role
                .role_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Failed to parse role_id: {}", e)))?;
            if !guild_roles.contains_key(&RoleId::new(role_id)) {
                role_repo.delete(role_id).await?;
                tracing::info!("Deleted role {} from guild {}", role_id, guild_id);
            }
        }

        // Upsert all current roles
        role_repo.upsert_many(guild_id, guild_roles).await?;

        tracing::info!("Updated {} roles for guild {}", guild_roles.len(), guild_id);

        Ok(())
    }

    /// Get paginated roles for a guild
    pub async fn get_paginated(
        &self,
        guild_id: u64,
        page: u64,
        entries: u64,
    ) -> Result<PaginatedDiscordGuildRolesDto, AppError> {
        use entity::prelude::DiscordGuildRole;
        use sea_orm::ColumnTrait;
        use sea_orm::QueryFilter;

        let paginator = DiscordGuildRole::find()
            .filter(entity::discord_guild_role::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::discord_guild_role::Column::Position)
            .paginate(self.db, entries);

        let total = paginator.num_pages().await?;
        let roles = paginator.fetch_page(page).await?;

        let role_dtos: Result<Vec<DiscordGuildRoleDto>, AppError> = roles
            .into_iter()
            .map(|role| {
                let guild_id = role.guild_id.parse::<u64>().map_err(|e| {
                    AppError::InternalError(format!("Failed to parse guild_id: {}", e))
                })?;
                let role_id = role.role_id.parse::<u64>().map_err(|e| {
                    AppError::InternalError(format!("Failed to parse role_id: {}", e))
                })?;
                Ok(DiscordGuildRoleDto {
                    id: role.id,
                    guild_id,
                    role_id,
                    name: role.name,
                    color: role.color,
                    position: role.position,
                })
            })
            .collect();

        Ok(PaginatedDiscordGuildRolesDto {
            roles: role_dtos?,
            total: total * entries,
            page,
            entries,
        })
    }
}
