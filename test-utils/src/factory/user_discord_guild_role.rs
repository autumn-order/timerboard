//! Factory for creating user Discord guild role test data.
//!
//! Provides factory methods for creating user-guild-role relationships with sensible defaults.
//! These relationships must have existing users and roles due to foreign key constraints.

use crate::fixture;
use entity::user_discord_guild_role;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for building user Discord guild role entities with custom values.
///
/// Allows customization of all fields before creation. Use `create_user_guild_role()`
/// for quick creation with defaults. Default values are sourced from the
/// user_discord_guild_role fixture for consistency across tests.
pub struct UserDiscordGuildRoleFactory<'a> {
    db: &'a DatabaseConnection,
    entity: user_discord_guild_role::Model,
}

impl<'a> UserDiscordGuildRoleFactory<'a> {
    /// Creates a new factory instance with default values from fixture.
    ///
    /// Defaults are sourced from `fixture::user_discord_guild_role::entity()`.
    /// The user_id and role_id are set to the provided values.
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    /// - `user_id` - Discord user ID
    /// - `role_id` - Discord role ID
    pub fn new(db: &'a DatabaseConnection, user_id: u64, role_id: u64) -> Self {
        let entity = fixture::user_discord_guild_role::entity_builder()
            .user_id(user_id.to_string())
            .role_id(role_id.to_string())
            .build();

        Self { db, entity }
    }

    /// Builds and inserts the user Discord guild role entity.
    ///
    /// # Returns
    /// - `Ok(Model)` - The created user guild role entity
    /// - `Err(DbErr)` - Database error during insertion
    pub async fn build(self) -> Result<user_discord_guild_role::Model, DbErr> {
        user_discord_guild_role::ActiveModel {
            user_id: ActiveValue::Set(self.entity.user_id),
            role_id: ActiveValue::Set(self.entity.role_id),
        }
        .insert(self.db)
        .await
    }
}

/// Creates a user Discord guild role relationship with default values.
///
/// Quick convenience function for creating a user-guild-role relationship without customization.
///
/// # Arguments
/// - `db` - Database connection for inserting the entity
/// - `user_id` - Discord user ID
/// - `role_id` - Discord role ID
///
/// # Returns
/// - `Ok(Model)` - The created user guild role entity
/// - `Err(DbErr)` - Database error during insertion
///
/// # Example
/// ```rust,ignore
/// let relationship = factory::user_discord_guild_role::create_user_guild_role(&db, 123456789, 987654321).await?;
/// ```
pub async fn create_user_guild_role(
    db: &DatabaseConnection,
    user_id: u64,
    role_id: u64,
) -> Result<user_discord_guild_role::Model, DbErr> {
    UserDiscordGuildRoleFactory::new(db, user_id, role_id)
        .build()
        .await
}

/// Creates multiple user-guild-role relationships for a single user.
///
/// Convenience function for creating multiple role assignments for one user.
/// Useful for testing scenarios where users have multiple roles.
///
/// # Arguments
/// - `db` - Database connection for inserting the entities
/// - `user_id` - Discord user ID
/// - `role_ids` - Slice of Discord role IDs to assign to the user
///
/// # Returns
/// - `Ok(Vec<Model>)` - Vector of created user guild role entities
/// - `Err(DbErr)` - Database error during insertion
///
/// # Example
/// ```rust,ignore
/// let relationships = factory::user_discord_guild_role::create_user_guild_roles(
///     &db,
///     123456789,
///     &[111, 222, 333]
/// ).await?;
/// ```
pub async fn create_user_guild_roles(
    db: &DatabaseConnection,
    user_id: u64,
    role_ids: &[u64],
) -> Result<Vec<user_discord_guild_role::Model>, DbErr> {
    let mut results = Vec::new();
    for role_id in role_ids {
        let relationship = create_user_guild_role(db, user_id, *role_id).await?;
        results.push(relationship);
    }
    Ok(results)
}
