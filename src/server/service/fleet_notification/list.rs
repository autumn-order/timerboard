//! Upcoming fleets list management operations.
//!
//! This module provides functionality for posting and updating the "upcoming fleets" list message in Discord channels.
//! The list displays all upcoming non-hidden fleets for categories configured to post in a channel,
//! with links to fleet messages and relative timestamps.

use dioxus_logger::tracing;
use serenity::all::{ChannelId, CreateEmbed, CreateMessage, EditMessage, MessageId, Timestamp};

use crate::server::{
    data::{
        category::FleetCategoryRepository, channel_fleet_list::ChannelFleetListRepository,
        fleet::FleetRepository, fleet_message::FleetMessageRepository,
    },
    error::AppError,
    model::channel_fleet_list::UpsertChannelFleetListParam,
};

use super::FleetNotificationService;

impl<'a> FleetNotificationService<'a> {
    /// Posts or updates the upcoming fleets list for a channel.
    ///
    /// Creates or updates a single message displaying all upcoming non-hidden fleets
    /// for categories configured to post in the channel. The list includes links to
    /// fleet messages and relative timestamps. Intelligently edits the existing list
    /// if it's still the most recent message, or deletes and reposts if other messages
    /// have been sent since. Uses Discord blurple color (0x5865F2).
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted/updated the upcoming fleets list
    /// - `Err(AppError::InternalError)` - Invalid channel or message ID format
    /// - `Err(AppError::Database)` - Database error retrieving fleets or categories
    pub async fn post_upcoming_fleets_list(&self, channel_id: u64) -> Result<(), AppError> {
        let channel_id_obj = ChannelId::new(channel_id);
        let now = chrono::Utc::now();

        let category_repo = FleetCategoryRepository::new(self.db);
        let fleet_repo = FleetRepository::new(self.db);
        let message_repo = FleetMessageRepository::new(self.db);
        let list_repo = ChannelFleetListRepository::new(self.db);

        // Get all categories that post to this channel
        let category_ids = category_repo
            .get_category_ids_by_channel(channel_id)
            .await?;

        if category_ids.is_empty() {
            tracing::debug!(
                "No categories configured for channel {}, skipping list update",
                channel_id
            );
            return Ok(());
        }

        // Get all upcoming fleets for these categories
        let fleets = fleet_repo
            .get_upcoming_by_categories(category_ids.clone(), now)
            .await?;

        if fleets.is_empty() {
            tracing::debug!("No upcoming fleets for channel {}", channel_id);
            // No upcoming fleets, optionally delete existing list message
            return Ok(());
        }

        // Get categories data for names
        let category_map = category_repo.get_names_by_ids(category_ids.clone()).await?;

        // Get guild_id from the first category for building message links
        let guild_id = if !category_ids.is_empty() {
            // Get one category to extract guild_id
            if let Some(category_data) = category_repo.find_by_id(category_ids[0]).await? {
                category_data.category.guild_id
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // Build description with bullet list of fleets
        let mut description = String::new();

        for fleet in fleets {
            let category_name = category_map
                .get(&fleet.category_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            // Get the most recent message for this fleet in this channel (prefer reminder over creation)
            let messages = message_repo
                .get_by_fleet_id_and_channel(fleet.id, channel_id)
                .await?;

            // Find reminder or creation message (not formup)
            let message_link = messages
                .iter()
                .filter(|m| m.message_type == "reminder" || m.message_type == "creation")
                .max_by_key(|m| &m.created_at)
                .map(|m| {
                    format!(
                        "https://discord.com/channels/{}/{}/{}",
                        guild_id, channel_id, m.message_id
                    )
                });

            if let Some(link) = message_link {
                // Format: • Category - [Fleet Name](link) - relative time
                let line = format!(
                    "• {} - [{}]({}) - <t:{}:R>\n",
                    category_name,
                    fleet.name,
                    link,
                    fleet.fleet_time.timestamp()
                );
                description.push_str(&line);
            }
        }

        // Build embed with description containing the fleet list
        let embed = CreateEmbed::new()
            .title(".:Upcoming Events:.")
            .url(&self.app_url)
            .description(description)
            .color(0x5865F2) // Discord blurple color
            .timestamp(Timestamp::from_unix_timestamp(now.timestamp()).unwrap());

        // Get or create the list message
        let existing_list = list_repo.get_by_channel_id(channel_id).await?;

        // Check if we should edit or post new message
        if let Some(existing) = existing_list {
            // Compare updated_at (when we posted the list) with last_message_at (most recent message in channel)
            // If our list message is still the most recent, edit it. Otherwise, delete and repost.
            let should_edit = existing.updated_at >= existing.last_message_at;

            tracing::debug!(
                "Channel {}: updated_at={}, last_message_at={}, should_edit={}",
                channel_id,
                existing.updated_at,
                existing.last_message_at,
                should_edit
            );

            if should_edit {
                // Edit the existing message since it's still the most recent
                self.edit_fleet_list_message(
                    channel_id_obj,
                    existing.message_id,
                    embed,
                    &list_repo,
                    channel_id,
                )
                .await?;
            } else {
                // Delete old message and post new one (to be most recent in channel)
                self.delete_and_repost_fleet_list(
                    channel_id_obj,
                    existing.message_id,
                    embed,
                    &list_repo,
                    channel_id,
                )
                .await?;
            }
        } else {
            // No existing list, post new message
            self.post_new_fleet_list_message(channel_id_obj, embed, &list_repo, channel_id)
                .await?;
        }

        Ok(())
    }

    /// Edits an existing fleet list message.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID object
    /// - `message_id` - Existing message ID to edit
    /// - `embed` - New embed to set
    /// - `list_repo` - Channel fleet list repository
    /// - `channel_id_u64` - Channel ID as u64 for repository operations
    ///
    /// # Returns
    /// - `Ok(())` - Successfully edited message
    /// - `Err(AppError)` - Database error
    async fn edit_fleet_list_message(
        &self,
        channel_id: ChannelId,
        message_id: u64,
        embed: CreateEmbed,
        list_repo: &ChannelFleetListRepository<'_>,
        channel_id_u64: u64,
    ) -> Result<(), AppError> {
        let edit_message = EditMessage::new().embed(embed);

        match self
            .http
            .edit_message(
                channel_id,
                MessageId::new(message_id),
                &edit_message,
                vec![],
            )
            .await
        {
            Ok(_) => {
                // Update the updated_at timestamp
                list_repo
                    .upsert(UpsertChannelFleetListParam {
                        channel_id: channel_id_u64,
                        message_id,
                    })
                    .await?;
                tracing::info!(
                    "Edited existing upcoming fleets list in channel {}",
                    channel_id_u64
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to edit upcoming fleets list in channel {}: {}",
                    channel_id_u64,
                    e
                );
            }
        }

        Ok(())
    }

    /// Posts a new fleet list message.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID object
    /// - `embed` - Embed to post
    /// - `list_repo` - Channel fleet list repository
    /// - `channel_id_u64` - Channel ID as u64 for repository operations
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted message
    /// - `Err(AppError)` - Database error
    async fn post_new_fleet_list_message(
        &self,
        channel_id: ChannelId,
        embed: CreateEmbed,
        list_repo: &ChannelFleetListRepository<'_>,
        channel_id_u64: u64,
    ) -> Result<(), AppError> {
        let new_message = CreateMessage::new().embed(embed);

        match channel_id.send_message(&self.http, new_message).await {
            Ok(msg) => {
                list_repo
                    .upsert(UpsertChannelFleetListParam {
                        channel_id: channel_id_u64,
                        message_id: msg.id.get(),
                    })
                    .await?;
                tracing::info!(
                    "Posted new upcoming fleets list in channel {}",
                    channel_id_u64
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to post upcoming fleets list in channel {}: {}",
                    channel_id_u64,
                    e
                );
            }
        }

        Ok(())
    }

    /// Deletes an old fleet list message and posts a new one.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID object
    /// - `old_message_id` - Old message ID to delete
    /// - `embed` - New embed to post
    /// - `list_repo` - Channel fleet list repository
    /// - `channel_id_u64` - Channel ID as u64 for repository operations
    ///
    /// # Returns
    /// - `Ok(())` - Successfully deleted and posted
    /// - `Err(AppError)` - Database error
    async fn delete_and_repost_fleet_list(
        &self,
        channel_id: ChannelId,
        old_message_id: u64,
        embed: CreateEmbed,
        list_repo: &ChannelFleetListRepository<'_>,
        channel_id_u64: u64,
    ) -> Result<(), AppError> {
        // Delete old message
        match self
            .http
            .delete_message(channel_id, MessageId::new(old_message_id), None)
            .await
        {
            Ok(_) => {
                tracing::debug!(
                    "Deleted old upcoming fleets list in channel {} (not most recent)",
                    channel_id_u64
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to delete old upcoming fleets list in channel {}: {}",
                    channel_id_u64,
                    e
                );
                // Continue anyway to post new message
            }
        }

        // Post new message
        self.post_new_fleet_list_message(channel_id, embed, list_repo, channel_id_u64)
            .await
    }
}
