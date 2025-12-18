//! Discord guild role repository for database operations.
//!
//! This module provides the `DiscordGuildRoleRepository` for managing Discord guild
//! roles in the database. It handles upserting roles from Discord, deleting removed
//! roles, and querying roles by guild. Role data is synced from Discord via Serenity
//! and stored locally for permission checks and display purposes.
//!
//! All methods return domain models at the repository boundary, converting SeaORM
//! entity models internally to prevent database-specific structures from leaking
//! into service and controller layers.

use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::{Role, RoleId};
use std::collections::HashMap;

use crate::server::model::discord::DiscordGuildRole;

/// Repository for Discord guild role database operations.
///
/// Provides methods for upserting, deleting, and querying Discord roles.
/// Used to keep local role data synchronized with Discord's state for
/// permission checks and UI display.
pub struct DiscordGuildRoleRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRoleRepository<'a> {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    /// - `db` - Database connection for executing queries
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Upserts a Discord guild role (insert or update if exists).
    ///
    /// Creates a new role record or updates an existing one based on role_id.
    /// Updates name, color, and position if the role already exists. This is
    /// used when syncing roles from Discord to keep local data current.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID that owns this role
    /// - `role` - Serenity role object containing role data from Discord
    ///
    /// # Returns
    /// - `Ok(DiscordGuildRole)` - The upserted role as a domain model
    /// - `Err(DbErr)` - Database error during upsert operation
    pub async fn upsert(&self, guild_id: u64, role: &Role) -> Result<DiscordGuildRole, DbErr> {
        let entity =
            entity::prelude::DiscordGuildRole::insert(entity::discord_guild_role::ActiveModel {
                guild_id: ActiveValue::Set(guild_id.to_string()),
                role_id: ActiveValue::Set(role.id.get().to_string()),
                name: ActiveValue::Set(role.name.clone()),
                color: ActiveValue::Set(format!("#{:06X}", role.colour.0)),
                position: ActiveValue::Set(role.position as i16),
            })
            .on_conflict(
                OnConflict::column(entity::discord_guild_role::Column::RoleId)
                    .update_columns([
                        entity::discord_guild_role::Column::Name,
                        entity::discord_guild_role::Column::Color,
                        entity::discord_guild_role::Column::Position,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(self.db)
            .await?;

        DiscordGuildRole::from_entity(entity)
    }

    /// Upserts multiple Discord guild roles in batch.
    ///
    /// Creates or updates multiple role records from a Discord role hashmap.
    /// Iterates through all roles and upserts each one individually. Used when
    /// syncing all roles for a guild from Discord.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID that owns these roles
    /// - `roles` - HashMap of Discord roles from Serenity (role_id -> role)
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuildRoleParam>)` - Vector of upserted roles as param models
    /// - `Err(DbErr)` - Database error during any upsert operation
    pub async fn upsert_many(
        &self,
        guild_id: u64,
        roles: &HashMap<RoleId, Role>,
    ) -> Result<Vec<DiscordGuildRole>, DbErr> {
        let mut results = Vec::new();

        for role in roles.values() {
            let param = self.upsert(guild_id, role).await?;
            results.push(param);
        }

        Ok(results)
    }

    /// Deletes a Discord guild role by role ID.
    ///
    /// Removes the role record from the database. This is called when a role
    /// is deleted in Discord. Related records (access roles, ping roles, user
    /// role memberships) are automatically deleted via database cascade constraints.
    ///
    /// # Arguments
    /// - `role_id` - Discord role ID to delete
    ///
    /// # Returns
    /// - `Ok(())` - Role successfully deleted (or didn't exist)
    /// - `Err(DbErr)` - Database error during deletion
    pub async fn delete(&self, role_id: u64) -> Result<(), DbErr> {
        entity::prelude::DiscordGuildRole::delete_many()
            .filter(entity::discord_guild_role::Column::RoleId.eq(role_id.to_string()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Gets all roles for a Discord guild.
    ///
    /// Retrieves all role records associated with the specified guild. Used for
    /// displaying available roles in permission configuration UIs and for checking
    /// what roles exist in a guild.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID to get roles for
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuildRoleParam>)` - Vector of roles in the guild
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_by_guild_id(&self, guild_id: u64) -> Result<Vec<DiscordGuildRole>, DbErr> {
        let entities = entity::prelude::DiscordGuildRole::find()
            .filter(entity::discord_guild_role::Column::GuildId.eq(guild_id.to_string()))
            .all(self.db)
            .await?;

        entities
            .into_iter()
            .map(DiscordGuildRole::from_entity)
            .collect()
    }
}
