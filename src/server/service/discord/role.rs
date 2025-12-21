//! Discord guild role service for managing guild role synchronization.
//!
//! This module provides the `DiscordGuildRoleService` for synchronizing Discord guild roles
//! with the database. It handles bulk role updates during bot startup and provides paginated
//! queries for role data used in the UI.

use dioxus_logger::tracing;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use serenity::all::Role;

use crate::{
    model::discord::{DiscordGuildRoleDto, PaginatedDiscordGuildRolesDto},
    server::{
        data::discord::DiscordGuildRoleRepository, error::AppError,
        util::parse::parse_u64_from_string,
    },
};

/// Service for managing Discord guild roles.
///
/// Provides methods for synchronizing role data from Discord's API to the database
/// and querying role information for display in the UI. Acts as the orchestration
/// layer between Discord bot events and the role repository.
pub struct DiscordGuildRoleService<'a> {
    /// Database connection for repository operations.
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRoleService<'a> {
    /// Creates a new DiscordGuildRoleService instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `DiscordGuildRoleService` - New service instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Updates roles for a guild by syncing with Discord's current state.
    ///
    /// Performs a complete sync of guild roles by deleting roles that no longer exist
    /// in Discord and upserting all current roles. This ensures the database accurately
    /// reflects Discord's role structure. Used during bot startup and when significant
    /// role changes occur in the guild.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID to update roles for
    /// - `guild_roles` - Slice of current Discord roles from the API
    ///
    /// # Returns
    /// - `Ok(())` - Roles synced successfully
    /// - `Err(AppError::Database)` - Database error during deletion or upsert
    pub async fn update_roles(&self, guild_id: u64, guild_roles: &[Role]) -> Result<(), AppError> {
        let role_repo = DiscordGuildRoleRepository::new(self.db);

        // Get existing roles from database
        let existing_roles = role_repo.get_by_guild_id(guild_id).await?;

        // Find roles that no longer exist in Discord and delete them
        for existing_role in existing_roles {
            let exists = guild_roles
                .iter()
                .any(|role| role.id.get() == existing_role.role_id);

            if !exists {
                role_repo.delete(existing_role.role_id).await?;
                tracing::info!(
                    "Deleted role {} from guild {}",
                    existing_role.role_id,
                    guild_id
                );
            }
        }

        // Upsert all current roles
        role_repo.upsert_many(guild_id, &guild_roles).await?;

        tracing::info!("Updated {} roles for guild {}", guild_roles.len(), guild_id);

        Ok(())
    }

    /// Gets paginated roles for a guild.
    ///
    /// Retrieves a paginated list of roles for the specified guild, ordered by position
    /// (Discord's role hierarchy). Converts database models to DTOs for API responses.
    /// Used for displaying role lists in the UI and role selection interfaces.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID to fetch roles for
    /// - `page` - Zero-based page number
    /// - `entries` - Number of roles per page
    ///
    /// # Returns
    /// - `Ok(PaginatedDiscordGuildRolesDto)` - Paginated role list with metadata
    /// - `Err(AppError::Database)` - Database error during fetch
    /// - `Err(AppError::InternalError)` - Failed to parse guild_id or role_id
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
                let guild_id = parse_u64_from_string(role.guild_id)?;
                let role_id = parse_u64_from_string(role.role_id)?;

                Ok(DiscordGuildRoleDto {
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
