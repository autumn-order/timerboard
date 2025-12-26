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
    /// fleet messages and relative timestamps. Only posts a new message if there have
    /// been no messages in the channel for the last 10 minutes. Otherwise, edits the
    /// existing message. Uses Discord blurple color (0x5865F2).
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
        let list_repo = ChannelFleetListRepository::new(self.db);
        let now = chrono::Utc::now();

        // Build the fleet list embed (returns None if no fleets and allow_empty=false)
        let Some(embed) = self.build_fleet_list_embed(channel_id, false).await? else {
            tracing::debug!("No upcoming fleets for channel {}", channel_id);
            return Ok(());
        };

        // Get or create the list message
        let existing_list = list_repo.get_by_channel_id(channel_id).await?;

        // Check if we should edit or post new message
        if let Some(existing) = existing_list {
            // Check if the fleet list is still the most recent message
            let is_most_recent = existing.updated_at >= existing.last_message_at;

            // Calculate time since last message in channel
            let time_since_last_message = now - existing.last_message_at;
            let ten_minutes = chrono::Duration::minutes(10);

            // Only post new message if:
            // 1. Fleet list is NOT the most recent message, AND
            // 2. No messages in channel for last 10 minutes
            // Otherwise, always edit the existing message
            let should_post_new = !is_most_recent && time_since_last_message >= ten_minutes;

            tracing::debug!(
                "Channel {}: is_most_recent={}, last_message_at={}, time_since={} mins, should_post_new={}",
                channel_id,
                is_most_recent,
                existing.last_message_at,
                time_since_last_message.num_minutes(),
                should_post_new
            );

            if should_post_new {
                // Channel has been quiet for 10+ minutes AND list is buried, delete old and post new
                self.delete_and_repost_fleet_list(
                    channel_id_obj,
                    existing.message_id,
                    embed,
                    &list_repo,
                    channel_id,
                )
                .await?;
            } else {
                // Fleet list is still most recent OR channel is active, just edit the existing message
                self.edit_fleet_list_message(
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

    /// Updates the upcoming fleets list by only editing the existing message.
    ///
    /// This method is used when a fleet is updated or deleted to reflect changes
    /// in the upcoming fleets list without bumping it to the most recent message.
    /// If there are no upcoming fleets, it displays "No upcoming fleets..." message.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID
    ///
    /// # Returns
    /// - `Ok(())` - Successfully updated the upcoming fleets list
    /// - `Err(AppError::InternalError)` - Invalid channel or message ID format
    /// - `Err(AppError::Database)` - Database error retrieving fleets or categories
    pub async fn update_upcoming_fleets_list(&self, channel_id: u64) -> Result<(), AppError> {
        let channel_id_obj = ChannelId::new(channel_id);
        let list_repo = ChannelFleetListRepository::new(self.db);

        // Build the fleet list embed (allow_empty=true to show "No upcoming fleets...")
        let Some(embed) = self.build_fleet_list_embed(channel_id, true).await? else {
            tracing::debug!(
                "No categories configured for channel {}, skipping list update",
                channel_id
            );
            return Ok(());
        };

        // Get the existing list message
        let existing_list = list_repo.get_by_channel_id(channel_id).await?;

        if let Some(existing) = existing_list {
            // Always edit the existing message, never bump to most recent
            self.edit_fleet_list_message(
                channel_id_obj,
                existing.message_id,
                embed,
                &list_repo,
                channel_id,
            )
            .await?;
            tracing::debug!(
                "Updated upcoming fleets list in channel {} (edit only, no bump)",
                channel_id
            );
        } else {
            // No existing list, post new message
            self.post_new_fleet_list_message(channel_id_obj, embed, &list_repo, channel_id)
                .await?;
        }

        Ok(())
    }

    /// Builds the fleet list embed with all upcoming fleets for a channel.
    ///
    /// Fetches categories, fleets, and builds the embed description with fleet links.
    ///
    /// # Arguments
    /// - `channel_id` - Discord channel ID
    /// - `allow_empty` - If true, returns embed with "No upcoming fleets..." when no fleets.
    ///                   If false, returns None when no fleets.
    ///
    /// # Returns
    /// - `Ok(Some(CreateEmbed))` - Successfully built embed
    /// - `Ok(None)` - No categories configured OR (no fleets and !allow_empty)
    /// - `Err(AppError)` - Database error
    async fn build_fleet_list_embed(
        &self,
        channel_id: u64,
        allow_empty: bool,
    ) -> Result<Option<CreateEmbed>, AppError> {
        let now = chrono::Utc::now();

        let category_repo = FleetCategoryRepository::new(self.db);
        let fleet_repo = FleetRepository::new(self.db);
        let message_repo = FleetMessageRepository::new(self.db);

        // Get all categories that post to this channel
        let category_ids = category_repo
            .get_category_ids_by_channel(channel_id)
            .await?;

        if category_ids.is_empty() {
            return Ok(None);
        }

        // Get all upcoming fleets for these categories
        let fleets = fleet_repo
            .get_upcoming_by_categories(category_ids.clone(), now)
            .await?;

        // Get guild_id from the first category for building message links
        let guild_id =
            if let Some(category_data) = category_repo.find_by_id(category_ids[0]).await? {
                category_data.category.guild_id
            } else {
                return Ok(None);
            };

        // Build description with bullet list of fleets
        let description = if fleets.is_empty() {
            if !allow_empty {
                return Ok(None);
            }
            "No upcoming events scheduled.".to_string()
        } else {
            // Get categories data for names
            let category_map = category_repo.get_names_by_ids(category_ids.clone()).await?;

            let mut desc = String::new();

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
                    desc.push_str(&line);
                }
            }

            desc
        };

        // Build embed with description containing the fleet list
        let embed = CreateEmbed::new()
            .title(".:Upcoming Events:.")
            .url(&self.app_url)
            .description(description)
            .color(0x5865F2) // Discord blurple color
            .timestamp(Timestamp::from_unix_timestamp(now.timestamp()).unwrap());

        Ok(Some(embed))
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
