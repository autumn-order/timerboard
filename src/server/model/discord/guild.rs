//! Discord guild domain models.
//!
//! Provides the domain model for Discord guilds (servers), tracking guild identity,
//! display properties, and synchronization metadata. Handles conversion between entity
//! models from the database and domain models used in the service layer.

use chrono::{DateTime, Utc};
use sea_orm::DbErr;

/// Discord server (guild) with display properties and sync metadata.
///
/// Tracks guild name, icon, and when the guild's roles, channels, and members
/// were last synchronized from Discord's API.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuild {
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Guild display name.
    pub name: String,
    /// Optional guild icon hash for constructing icon URLs.
    pub icon_hash: Option<String>,
    /// Timestamp of the last full guild sync (roles, channels, members).
    pub last_sync_at: DateTime<Utc>,
}

impl DiscordGuild {
    /// Converts an entity model to a domain model at the repository boundary.
    ///
    /// Parses the string guild_id from the database into u64 for type-safe
    /// usage in the service layer. Also preserves guild display properties
    /// and sync metadata.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuild)` - Successfully converted domain model
    /// - `Err(DbErr::Custom)` - Failed to parse guild_id as u64
    pub fn from_entity(entity: entity::discord_guild::Model) -> Result<Self, DbErr> {
        let guild_id = entity
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            guild_id,
            name: entity.name,
            icon_hash: entity.icon_hash,
            last_sync_at: entity.last_sync_at,
        })
    }
}
