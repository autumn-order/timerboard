use dioxus_logger::tracing;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use serenity::all::{ChannelId, ChannelType, GuildChannel};
use std::collections::HashMap;

use crate::{
    model::discord::{DiscordGuildChannelDto, PaginatedDiscordGuildChannelsDto},
    server::{data::discord::DiscordGuildChannelRepository, error::AppError},
};

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
            let channel_id = existing_channel.channel_id.parse::<u64>().map_err(|e| {
                AppError::InternalError(format!("Failed to parse channel_id: {}", e))
            })?;
            if !text_channels.contains_key(&ChannelId::new(channel_id)) {
                channel_repo.delete(channel_id).await?;
                tracing::info!("Deleted channel {} from guild {}", channel_id, guild_id);
            }
        }

        // Upsert all current text channels
        for channel in text_channels.values() {
            channel_repo.upsert(guild_id, channel).await?;
        }

        tracing::info!(
            "Updated {} text channels for guild {}",
            text_channels.len(),
            guild_id
        );

        Ok(())
    }

    /// Get paginated channels for a guild
    pub async fn get_paginated(
        &self,
        guild_id: u64,
        page: u64,
        entries: u64,
    ) -> Result<PaginatedDiscordGuildChannelsDto, AppError> {
        use entity::prelude::DiscordGuildChannel;
        use sea_orm::ColumnTrait;
        use sea_orm::QueryFilter;

        let paginator = DiscordGuildChannel::find()
            .filter(entity::discord_guild_channel::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::discord_guild_channel::Column::Position)
            .paginate(self.db, entries);

        let total = paginator.num_pages().await?;
        let channels = paginator.fetch_page(page).await?;

        let channel_dtos: Result<Vec<DiscordGuildChannelDto>, AppError> = channels
            .into_iter()
            .map(|channel| {
                let guild_id = channel.guild_id.parse::<u64>().map_err(|e| {
                    AppError::InternalError(format!("Failed to parse guild_id: {}", e))
                })?;
                let channel_id = channel.channel_id.parse::<u64>().map_err(|e| {
                    AppError::InternalError(format!("Failed to parse channel_id: {}", e))
                })?;
                Ok(DiscordGuildChannelDto {
                    guild_id,
                    channel_id,
                    name: channel.name,
                    position: channel.position,
                })
            })
            .collect();

        Ok(PaginatedDiscordGuildChannelsDto {
            channels: channel_dtos?,
            total: total * entries,
            page,
            entries,
        })
    }
}
