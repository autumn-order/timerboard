//! Fleet formup notification operations.
//!
//! This module provides functionality for posting formup (start) notifications for fleets at their start time.
//! Formup messages are posted as replies to the most recent existing messages (reminder or creation).

use dioxus_logger::tracing;
use serenity::all::{ChannelId, CreateEmbed, CreateMessage, MessageId, MessageReference};

use crate::server::{
    data::fleet_message::FleetMessageRepository,
    error::AppError,
    model::{
        category::FleetCategoryWithRelations,
        fleet::Fleet,
        fleet_message::{CreateFleetMessageParam, FleetMessage},
    },
    util::parse::parse_u64_from_string,
};

use super::FleetNotificationService;

impl<'a> FleetNotificationService<'a> {
    /// Posts fleet formup message as a reply to existing fleet messages.
    ///
    /// Creates formup notifications at fleet time to signal immediate gathering. Replies
    /// to the most recent existing message (reminder or creation) for each channel.
    /// Uses red embed color (0xe74c3c) to indicate urgency.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted formup messages to all channels
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving or storing messages
    pub async fn post_fleet_formup(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        let message_repo = FleetMessageRepository::new(self.db);

        // Get existing messages to reply to
        let existing_messages = message_repo.get_by_fleet_id(fleet.id).await?;

        if existing_messages.is_empty() {
            tracing::warn!(
                "No existing messages found for fleet {}, skipping formup",
                fleet.id
            );
            return Ok(());
        }

        // Get category with channels and ping roles
        let (category_data, guild_id) = self
            .get_category_data_with_guild_id(fleet.category_id)
            .await?;

        // Get ping format fields for the category
        let fields = self
            .get_ping_format_fields(&category_data, guild_id)
            .await?;

        // Build embed with commander name
        let embed = self
            .build_fleet_embed_with_commander(
                fleet,
                &fields[..],
                field_values,
                0xe74c3c, // Red color for formup
                guild_id,
            )
            .await?;

        // Build title for formup
        let title = format!("**.:{} Forming Now:.**", category_data.category.name);

        // Build ping content with title
        let content = self.build_ping_content(&title, &category_data, guild_id)?;

        // Post to all configured channels
        self.post_formup_messages(
            fleet,
            &message_repo,
            &existing_messages,
            &category_data,
            &content,
            &embed,
        )
        .await
    }

    /// Posts fleet formup messages to all configured channels.
    ///
    /// # Arguments
    /// - `fleet` - Fleet data
    /// - `message_repo` - Fleet message repository
    /// - `existing_messages` - Existing messages for reference replies
    /// - `category_data` - Category data with channels
    /// - `content` - Message content with role pings
    /// - `embed` - Fleet embed to post
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted to all channels (or failed gracefully)
    /// - `Err(AppError)` - Critical error (database or parsing)
    async fn post_formup_messages(
        &self,
        fleet: &Fleet,
        message_repo: &FleetMessageRepository<'_>,
        existing_messages: &[FleetMessage],
        category_data: &FleetCategoryWithRelations,
        content: &str,
        embed: &CreateEmbed,
    ) -> Result<(), AppError> {
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = parse_u64_from_string(channel.channel_id.clone())?;
            let channel_id = ChannelId::new(channel_id_u64);

            // Find the most recent message for this channel (prefer reminder over creation)
            let reference_msg = existing_messages
                .iter()
                .filter(|m| m.channel_id == channel_id_u64)
                .max_by_key(|m| &m.created_at);

            let mut message = CreateMessage::new().content(content).embed(embed.clone());

            // Reply to the most recent message if it exists
            if let Some(ref_msg) = reference_msg {
                message = message.reference_message(MessageReference::from((
                    channel_id,
                    MessageId::new(ref_msg.message_id),
                )));
            }

            match channel_id.send_message(&self.http, message).await {
                Ok(msg) => {
                    // Store message in database
                    message_repo
                        .create(CreateFleetMessageParam {
                            fleet_id: fleet.id,
                            channel_id: channel_id_u64,
                            message_id: msg.id.get(),
                            message_type: "formup".to_string(),
                        })
                        .await?;

                    tracing::info!(
                        "Posted fleet formup for fleet {} to channel {}",
                        fleet.id,
                        channel_id_u64
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post fleet formup to channel {}: {}",
                        channel_id_u64,
                        e
                    );
                    // Continue posting to other channels even if one fails
                }
            }
        }

        Ok(())
    }
}
