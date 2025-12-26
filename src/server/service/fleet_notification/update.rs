//! Fleet notification update operations.
//!
//! This module provides functionality for updating existing fleet notifications.
//! It handles editing Discord messages with new fleet information.

use dioxus_logger::tracing;
use serenity::all::{ChannelId, CreateEmbed, EditMessage, MessageId};

use crate::server::{
    data::fleet_message::FleetMessageRepository,
    error::AppError,
    model::{fleet::Fleet, fleet_message::FleetMessage},
};

use super::FleetNotificationService;

impl<'a> FleetNotificationService<'a> {
    /// Updates all existing fleet messages with new fleet information.
    ///
    /// Edits all Discord messages associated with the fleet to reflect updated details.
    /// Continues updating remaining messages even if individual updates fail. Uses blue
    /// embed color (0x3498db) for updates. Logs errors for failed updates but doesn't
    /// propagate them to allow partial success.
    ///
    /// # Arguments
    /// - `fleet` - Updated fleet domain model with current event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully updated all messages (or no messages exist)
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving messages or fields
    pub async fn update_fleet_messages(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        let message_repo = FleetMessageRepository::new(self.db);

        // Get all existing messages for this fleet
        let messages = message_repo.get_by_fleet_id(fleet.id).await?;

        if messages.is_empty() {
            tracing::debug!("No messages found for fleet {}, skipping update", fleet.id);
            return Ok(());
        }

        // Get category data and guild_id
        let (category_data, guild_id) = self
            .get_category_data_with_guild_id(fleet.category_id)
            .await?;

        // Get ping format fields
        let fields = self
            .get_ping_format_fields(&category_data, guild_id)
            .await?;

        // Build updated embed with commander name (use blue color for updates)
        let embed = self
            .build_fleet_embed_with_commander(fleet, &fields[..], field_values, 0x3498db, guild_id)
            .await?;

        // Update each message
        self.update_existing_messages(&messages, &embed).await
    }

    /// Updates existing fleet messages with new embed.
    ///
    /// # Arguments
    /// - `messages` - Existing fleet messages to update
    /// - `embed` - New fleet embed to set
    ///
    /// # Returns
    /// - `Ok(())` - Successfully updated all messages (or failed gracefully)
    async fn update_existing_messages(
        &self,
        messages: &[FleetMessage],
        embed: &CreateEmbed,
    ) -> Result<(), AppError> {
        for message in messages {
            let channel_id = ChannelId::new(message.channel_id);
            let msg_id = MessageId::new(message.message_id);

            let edit_builder = EditMessage::new().embed(embed.clone());

            match self
                .http
                .edit_message(channel_id, msg_id, &edit_builder, vec![])
                .await
            {
                Ok(_) => {
                    tracing::info!("Updated fleet message {} in channel {}", msg_id, channel_id);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to update fleet message {} in channel {}: {}",
                        msg_id,
                        channel_id,
                        e
                    );
                    // Continue updating other messages even if one fails
                }
            }
        }

        Ok(())
    }
}
