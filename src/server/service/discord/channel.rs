use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
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

    /// Updates channels for a guild by deleting channels that no longer exist and upserting current text channels.
    ///
    /// Filters for text channels only (excludes voice, category, forum, etc.).
    /// Removes channels from the database that no longer exist in Discord.
    /// Creates or updates all current text channels.
    ///
    /// # Arguments
    /// - `guild_id` - Discord's unique identifier for the guild
    /// - `guild_channels` - HashMap of all channels in the guild
    ///
    /// # Returns
    /// - `Ok(())` - Channels updated successfully
    /// - `Err(AppError)` - Database error during sync
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
            if !text_channels.contains_key(&ChannelId::new(existing_channel.channel_id)) {
                channel_repo.delete(existing_channel.channel_id).await?;
                tracing::info!(
                    "Deleted channel {} from guild {}",
                    existing_channel.channel_id,
                    guild_id
                );
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

    /// Retrieves paginated channels for a guild.
    ///
    /// Fetches channels from the database with pagination support, converting
    /// domain models to DTOs for API responses. Uses a simple offset-based
    /// pagination approach.
    ///
    /// # Arguments
    /// - `guild_id` - Discord's unique identifier for the guild
    /// - `page` - Page number (0-indexed)
    /// - `entries` - Number of entries per page
    ///
    /// # Returns
    /// - `Ok(PaginatedDiscordGuildChannelsDto)` - Paginated channel data with metadata
    /// - `Err(AppError)` - Database error during query or entity conversion failure
    pub async fn get_paginated(
        &self,
        guild_id: u64,
        page: u64,
        entries: u64,
    ) -> Result<PaginatedDiscordGuildChannelsDto, AppError> {
        let channel_repo = DiscordGuildChannelRepository::new(self.db);

        // Get all channels for the guild (already sorted by position)
        let all_channels = channel_repo.get_by_guild_id(guild_id).await?;

        // Calculate pagination
        let total = all_channels.len() as u64;
        let start = (page * entries) as usize;

        // Get the page slice and convert to DTOs
        let channel_dtos: Vec<DiscordGuildChannelDto> = all_channels
            .into_iter()
            .skip(start)
            .take(entries as usize)
            .map(|channel| channel.into_dto())
            .collect();

        Ok(PaginatedDiscordGuildChannelsDto {
            channels: channel_dtos,
            total,
            page,
            entries,
        })
    }
}
