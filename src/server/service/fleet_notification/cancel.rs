//! Fleet notification cancellation operations.
//!
//! This module provides functionality for cancelling fleet messages by editing them with cancellation notices.
//! Cancellation messages display that the fleet has been cancelled with relevant metadata.

use dioxus_logger::tracing;
use serenity::all::{ChannelId, CreateEmbed, EditMessage, MessageId, Timestamp};

use crate::server::{
    data::fleet_message::FleetMessageRepository,
    error::{internal::InternalError, AppError},
    model::{fleet::Fleet, fleet_message::FleetMessage},
};

use super::{builder, FleetNotificationService};

impl<'a> FleetNotificationService<'a> {
    /// Cancels all existing fleet messages by editing them with cancellation notice.
    ///
    /// Edits all Discord messages associated with the fleet to display cancellation
    /// information. Uses gray embed color (0x95a5a6) and includes cancellation timestamp
    /// and cancelled-by information. Continues cancelling remaining messages even if
    /// individual edits fail.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model being cancelled
    ///
    /// # Returns
    /// - `Ok(())` - Successfully cancelled all messages (or no messages exist)
    /// - `Err(AppError::NotFound)` - Fleet category not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving messages
    pub async fn cancel_fleet_messages(
        &self,
        fleet: &Fleet,
        app_url: &str,
    ) -> Result<(), AppError> {
        let message_repo = FleetMessageRepository::new(self.db);

        // Get all existing messages for this fleet
        let messages = message_repo.get_by_fleet_id(fleet.id).await?;

        if messages.is_empty() {
            tracing::debug!("No messages found for fleet {}, skipping cancel", fleet.id);
            return Ok(());
        }

        // Get category data and guild_id
        let (category_data, guild_id) = self
            .get_category_data_with_guild_id(fleet.category_id)
            .await?;

        // Fetch commander name from Discord
        let commander_name =
            builder::get_commander_name(self.http.clone(), fleet, guild_id).await?;

        // Build cancellation embed
        let now = chrono::Utc::now();
        let timestamp = Timestamp::from_unix_timestamp(now.timestamp()).map_err(|e| {
            AppError::InternalError(InternalError::InvalidDiscordTimestamp {
                timestamp: now.timestamp(),
                reason: e.to_string(),
            })
        })?;

        let embed = CreateEmbed::new()
            .title(format!(".:{} Cancelled:.", category_data.category.name))
            .url(app_url)
            .color(0x95a5a6) // Gray color for cancellation
            .description(format!(
                "{} posted by <@{}>, **{}**, scheduled for **{} UTC** (<t:{}:F>) was cancelled.",
                category_data.category.name,
                fleet.commander_id,
                fleet.name,
                fleet.fleet_time.format("%Y-%m-%d %H:%M"),
                fleet.fleet_time.timestamp()
            ))
            .footer(serenity::all::CreateEmbedFooter::new(format!(
                "Cancelled by: {}",
                commander_name
            )))
            .timestamp(timestamp);

        // Update each message with cancellation notice
        self.cancel_existing_messages(&messages, &embed).await
    }

    /// Cancels existing fleet messages by editing them with cancellation embed.
    ///
    /// # Arguments
    /// - `messages` - Existing fleet messages to cancel
    /// - `embed` - Cancellation embed to set
    ///
    /// # Returns
    /// - `Ok(())` - Successfully cancelled all messages (or failed gracefully)
    async fn cancel_existing_messages(
        &self,
        messages: &[FleetMessage],
        embed: &CreateEmbed,
    ) -> Result<(), AppError> {
        for message in messages {
            let channel_id = ChannelId::new(message.channel_id);
            let msg_id = MessageId::new(message.message_id);

            // Clear content and set cancellation embed
            let edit_builder = EditMessage::new().content("").embed(embed.clone());

            match self
                .http
                .edit_message(channel_id, msg_id, &edit_builder, vec![])
                .await
            {
                Ok(_) => {
                    tracing::info!(
                        "Cancelled fleet message {} in channel {}",
                        msg_id,
                        channel_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to cancel fleet message {} in channel {}: {}",
                        msg_id,
                        channel_id,
                        e
                    );
                    // Continue cancelling other messages even if one fails
                }
            }
        }

        Ok(())
    }
}
