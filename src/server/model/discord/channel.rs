//! Discord guild channel domain models.
//!
//! Provides the domain model for Discord channels within guilds, tracking channel
//! identity, display properties, and position for ordering. Handles conversion between
//! entity models from the database and domain models used in the service layer.

use crate::model::discord::DiscordGuildChannelDto;
use sea_orm::DbErr;

/// Discord channel within a guild with display properties and hierarchy position.
///
/// Tracks channel name and position in the guild's channel list for display
/// ordering and management purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuildChannel {
    /// Discord channel ID as a u64.
    pub channel_id: u64,
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Channel display name.
    pub name: String,
    /// Channel position in the guild's channel list (for display ordering).
    pub position: i32,
}

impl DiscordGuildChannel {
    /// Converts an entity model to a domain model at the repository boundary.
    ///
    /// Parses string IDs from the database into u64 values for type safety.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuildChannel)` - Successfully converted domain model
    /// - `Err(DbErr::Custom)` - Failed to parse channel_id or guild_id as u64
    pub fn from_entity(entity: entity::discord_guild_channel::Model) -> Result<Self, DbErr> {
        let channel_id = entity
            .channel_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse channel_id: {}", e)))?;

        let guild_id = entity
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            channel_id,
            guild_id,
            name: entity.name,
            position: entity.position,
        })
    }

    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `DiscordGuildChannelDto` - DTO with all channel fields for serialization
    pub fn into_dto(self) -> DiscordGuildChannelDto {
        DiscordGuildChannelDto {
            guild_id: self.guild_id,
            channel_id: self.channel_id,
            name: self.name,
            position: self.position,
        }
    }
}
