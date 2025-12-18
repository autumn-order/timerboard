use chrono::{DateTime, Utc};
use sea_orm::DbErr;

/// Represents a Discord guild.
///
/// This param model represents a Discord server (guild) with its display
/// properties and sync metadata. It's used by the service layer to work
/// with Discord guilds without depending on database entity models.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuildParam {
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Guild display name.
    pub name: String,
    /// Optional guild icon hash for constructing icon URLs.
    pub icon_hash: Option<String>,
    /// Timestamp of the last full guild sync (roles, channels, members).
    pub last_sync_at: DateTime<Utc>,
}

impl DiscordGuildParam {
    /// Converts an entity model to a param model at the repository boundary.
    ///
    /// Parses the string guild_id from the database into u64 for type-safe
    /// usage in the service layer. Also preserves guild display properties
    /// and sync metadata.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuildParam)` - Successfully converted param model
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
