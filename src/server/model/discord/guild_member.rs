//! Discord guild member domain models.
//!
//! Provides the domain model for Discord users' guild memberships, tracking user identity
//! and guild-specific display properties like nicknames. Handles conversion between entity
//! models from the database and domain models used in the service layer.

use sea_orm::DbErr;

/// Discord user's membership in a guild with identity information.
///
/// Tracks the user's Discord username and optional guild-specific nickname
/// for display purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuildMember {
    /// Discord user ID as a u64.
    pub user_id: u64,
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Discord username.
    pub username: String,
    /// Optional guild-specific nickname.
    pub nickname: Option<String>,
}

impl DiscordGuildMember {
    /// Converts an entity model to a domain model at the repository boundary.
    ///
    /// Parses string IDs from the database into u64 values for type safety.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuildMember)` - Successfully converted domain model
    /// - `Err(DbErr::Custom)` - Failed to parse user_id or guild_id as u64
    pub fn from_entity(entity: entity::discord_guild_member::Model) -> Result<Self, DbErr> {
        let user_id = entity
            .user_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse user_id: {}", e)))?;

        let guild_id = entity
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            user_id,
            guild_id,
            username: entity.username,
            nickname: entity.nickname,
        })
    }
}
