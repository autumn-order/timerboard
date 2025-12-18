//! Discord guild role domain models.
//!
//! Provides the domain model for Discord roles within guilds, tracking role identity,
//! display properties, and hierarchy position. Handles conversion between entity models
//! from the database and domain models used in the service layer.

use sea_orm::DbErr;

/// Discord role within a guild with display properties and hierarchy position.
///
/// Tracks role name, color for display styling, and position in the guild's
/// role hierarchy where higher positions indicate greater importance.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuildRole {
    /// Discord role ID as a u64.
    pub role_id: u64,
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Role display name.
    pub name: String,
    /// Role color in hex format (e.g., "#FF5733").
    pub color: String,
    /// Role position in the guild's role hierarchy (higher = more important).
    pub position: i16,
}

impl DiscordGuildRole {
    /// Converts an entity model to a domain model at the repository boundary.
    ///
    /// Parses string IDs from the database into u64 values for type safety.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuildRole)` - Successfully converted domain model
    /// - `Err(DbErr::Custom)` - Failed to parse role_id or guild_id as u64
    pub fn from_entity(entity: entity::discord_guild_role::Model) -> Result<Self, DbErr> {
        let role_id = entity
            .role_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse role_id: {}", e)))?;

        let guild_id = entity
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            role_id,
            guild_id,
            name: entity.name,
            color: entity.color,
            position: entity.position,
        })
    }
}
