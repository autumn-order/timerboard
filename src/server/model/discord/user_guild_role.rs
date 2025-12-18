//! User-role association domain models for Discord guilds.
//!
//! Provides the domain model for the many-to-many relationship between Discord users
//! and roles within guilds. Handles conversion between entity models from the database
//! and domain models used in the service layer.

use sea_orm::DbErr;

/// User's membership in a Discord role within a guild.
///
/// Represents the many-to-many relationship between users and roles, tracking
/// which roles each user has been assigned in a specific guild.
#[derive(Debug, Clone, PartialEq)]
pub struct UserDiscordGuildRole {
    /// Discord user ID as a u64.
    pub user_id: u64,
    /// Discord role ID as a u64.
    pub role_id: u64,
}

impl UserDiscordGuildRole {
    /// Converts an entity model to a domain model at the repository boundary.
    ///
    /// Parses string IDs from the database into u64 values for type safety.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(UserDiscordGuildRole)` - Successfully converted domain model
    /// - `Err(DbErr::Custom)` - Failed to parse user_id or role_id as u64
    pub fn from_entity(entity: entity::user_discord_guild_role::Model) -> Result<Self, DbErr> {
        let user_id = entity
            .user_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse user_id: {}", e)))?;

        let role_id = entity
            .role_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse role_id: {}", e)))?;

        Ok(Self { user_id, role_id })
    }
}
