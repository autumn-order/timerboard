use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{ChannelId, ChannelType, GuildChannel};
use std::collections::HashMap;

use crate::server::{data::discord::DiscordGuildChannelRepository, error::AppError};

pub struct DiscordGuildChannelService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildChannelService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Updates channels for a guild by deleting channels that no longer exist and upserting current text channels
    ///
    /// Filters for text channels only (excludes voice, category, forum, etc.).
    /// Removes channels from the database that no longer exist in Discord.
    /// Creates or updates all current text channels.
    ///
    /// # Arguments
    /// - `guild_id`: Discord's unique identifier for the guild (u64)
    /// - `guild_channels`: HashMap of all channels in the guild
    ///
    /// # Returns
    /// - `Ok(())`: Channels updated successfully
    /// - `Err(AppError)`: Database error during sync
    pub async fn update_channels(
        &self,
        guild_id: u64,
        guild_channels: &HashMap<ChannelId, GuildChannel>,
    ) -> Result<(), AppError> {
        let channel_repo = DiscordGuildChannelRepository::new(self.db);

        // Filter for text channels only
        let text_channels: HashMap<ChannelId, &GuildChannel> = guild_channels
            .iter()
            .filter(|(_, channel)| channel.kind == ChannelType::Text)
            .map(|(id, channel)| (*id, channel))
            .collect();

        // Get existing channels from database
        let existing_channels = channel_repo.get_by_guild_id(guild_id).await?;

        // Find channels that no longer exist in Discord and delete them
        for existing_channel in existing_channels {
            let channel_id = existing_channel.channel_id as u64;
            if !text_channels.contains_key(&ChannelId::new(channel_id)) {
                channel_repo.delete(channel_id).await?;
                tracing::info!("Deleted channel {} from guild {}", channel_id, guild_id);
            }
        }

        // Upsert all current text channels
        for (_, channel) in &text_channels {
            channel_repo.upsert(guild_id, channel).await?;
        }

        tracing::info!(
            "Updated {} text channels for guild {}",
            text_channels.len(),
            guild_id
        );

        Ok(())
    }
}
