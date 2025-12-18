//! User Discord guild role repository for database operations.
//!
//! This module provides the `UserDiscordGuildRoleRepository` for managing the
//! many-to-many relationship between users and Discord guild roles. It handles
//! creating, deleting, and syncing role memberships as users gain or lose roles
//! in Discord guilds.
//!
//! All methods return domain models at the repository boundary, converting SeaORM
//! entity models internally to prevent database-specific structures from leaking
//! into service and controller layers.

use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::server::model::discord::UserDiscordGuildRole;

/// Repository for user Discord guild role relationship operations.
///
/// Provides methods for managing the many-to-many relationship between users
/// and Discord roles within guilds. Used for permission checks and role-based
/// access control throughout the application.
pub struct UserDiscordGuildRoleRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildRoleRepository<'a> {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    /// - `db` - Database connection for executing queries
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a relationship between a user and a guild role.
    ///
    /// Establishes that the specified user has the specified role in a guild.
    /// Does not check if the relationship already exists - will fail with a
    /// database error if attempting to create a duplicate.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `role_id` - Discord role ID as u64
    ///
    /// # Returns
    /// - `Ok(UserDiscordGuildRole)` - The created role membership record
    /// - `Err(DbErr)` - Database error during insert operation
    pub async fn create(&self, user_id: u64, role_id: u64) -> Result<UserDiscordGuildRole, DbErr> {
        let entity = entity::prelude::UserDiscordGuildRole::insert(
            entity::user_discord_guild_role::ActiveModel {
                user_id: ActiveValue::Set(user_id.to_string()),
                role_id: ActiveValue::Set(role_id.to_string()),
            },
        )
        .exec_with_returning(self.db)
        .await?;

        UserDiscordGuildRole::from_entity(entity)
    }

    /// Creates multiple user-guild-role relationships for a single user.
    ///
    /// Establishes relationships between the user and multiple guild roles. Checks for
    /// existing relationships before creating new ones to avoid duplicate key violations.
    /// Only returns newly created relationships - existing ones are silently skipped.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `role_ids` - Slice of Discord role IDs to assign to the user
    ///
    /// # Returns
    /// - `Ok(Vec<UserDiscordGuildRoleParam>)` - Vector of newly created relationships (excludes existing)
    /// - `Err(DbErr)` - Database error during query or insertion
    pub async fn create_many(
        &self,
        user_id: u64,
        role_ids: &[u64],
    ) -> Result<Vec<UserDiscordGuildRole>, DbErr> {
        let mut results = Vec::new();

        let user_id_str = user_id.to_string();
        for role_id in role_ids {
            // Check if relationship already exists
            let role_id_str = role_id.to_string();
            let exists = entity::prelude::UserDiscordGuildRole::find()
                .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id_str.as_str()))
                .filter(entity::user_discord_guild_role::Column::RoleId.eq(role_id_str.as_str()))
                .one(self.db)
                .await?;

            if exists.is_none() {
                let param = self.create(user_id, *role_id).await?;
                results.push(param);
            }
        }

        Ok(results)
    }

    /// Deletes all guild role relationships for a specific user.
    ///
    /// Removes all role memberships for the user across all guilds. Typically used
    /// when cleaning up user data or when re-syncing all of a user's roles from
    /// scratch (delete all, then re-insert current roles).
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    ///
    /// # Returns
    /// - `Ok(())` - All relationships successfully deleted (or none existed)
    /// - `Err(DbErr)` - Database error during deletion
    pub async fn delete_by_user(&self, user_id: u64) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuildRole::delete_many()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Deletes a specific user-guild-role relationship.
    ///
    /// Removes the relationship indicating that the user has the specified role.
    /// Called when a user loses a role in Discord (role removed event). No-op if
    /// the relationship doesn't exist.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `role_id` - Discord role ID
    ///
    /// # Returns
    /// - `Ok(())` - Relationship successfully deleted (or didn't exist)
    /// - `Err(DbErr)` - Database error during deletion
    pub async fn delete(&self, user_id: u64, role_id: u64) -> Result<(), DbErr> {
        let user_id_str = user_id.to_string();
        let role_id_str = role_id.to_string();
        entity::prelude::UserDiscordGuildRole::delete_many()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id_str.as_str()))
            .filter(entity::user_discord_guild_role::Column::RoleId.eq(role_id_str.as_str()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Syncs user's guild role memberships by replacing all existing roles.
    ///
    /// Replaces all role memberships for a user with the provided list. This is a
    /// two-step operation: first deletes all existing role relationships, then creates
    /// new ones for the provided Discord role IDs. Used when fetching a user's current
    /// roles from Discord to ensure local state matches Discord's state.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `role_ids` - Slice of Discord role IDs the user should have
    ///
    /// # Returns
    /// - `Ok(())` - Sync completed successfully
    /// - `Err(DbErr)` - Database error during deletion or creation operations
    pub async fn sync_user_roles(&self, user_id: u64, role_ids: &[u64]) -> Result<(), DbErr> {
        // Delete all existing role relationships for this user
        self.delete_by_user(user_id).await?;

        // Create new relationships
        self.create_many(user_id, role_ids).await?;

        Ok(())
    }
}
