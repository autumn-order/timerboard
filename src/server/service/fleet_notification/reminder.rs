//! Fleet reminder notification operations.
//!
//! This module provides functionality for posting reminder notifications for fleets before they start.
//! Reminder messages are posted as replies to creation messages when they exist,
//! or as standalone messages if the fleet was initially hidden.

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
    /// Posts fleet reminder message as a reply to the creation message.
    ///
    /// Creates reminder notifications before fleet time to alert participants. If creation
    /// messages exist, replies to them. If the fleet was initially hidden (no creation
    /// messages), posts as new messages. Skips posting if `disable_reminder` is true.
    /// Uses orange embed color (0xf39c12).
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted reminder messages or skipped (if disabled)
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving or storing messages
    pub async fn post_fleet_reminder(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        // Skip if reminders are disabled for this fleet
        if fleet.disable_reminder {
            tracing::debug!("Reminder disabled for fleet {}, skipping", fleet.id);
            return Ok(());
        }

        let message_repo = FleetMessageRepository::new(self.db);

        // Get existing creation messages to determine if we should reply or post new
        let creation_messages = message_repo.get_by_fleet_id(fleet.id).await?;

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
                0xf39c12, // Orange color for reminder
                guild_id,
            )
            .await?;

        // Build title - if no creation messages exist, treat as creation
        let title = if creation_messages.is_empty() {
            format!("**.:New Upcoming {}:.**", category_data.category.name)
        } else {
            format!(
                "**.:Reminder - Upcoming {}:.**",
                category_data.category.name
            )
        };

        // Build ping content with title
        let content = self.build_ping_content(&title, &category_data, guild_id)?;

        // Post to all configured channels
        self.post_reminder_messages(
            fleet,
            &message_repo,
            &creation_messages,
            &category_data,
            &content,
            &embed,
        )
        .await
    }

    /// Posts fleet reminder messages to all configured channels.
    ///
    /// # Arguments
    /// - `fleet` - Fleet data
    /// - `message_repo` - Fleet message repository
    /// - `creation_messages` - Existing creation messages for reference replies
    /// - `category_data` - Category data with channels
    /// - `content` - Message content with role pings
    /// - `embed` - Fleet embed to post
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted to all channels (or failed gracefully)
    /// - `Err(AppError)` - Critical error (database or parsing)
    async fn post_reminder_messages(
        &self,
        fleet: &Fleet,
        message_repo: &FleetMessageRepository<'_>,
        creation_messages: &[FleetMessage],
        category_data: &FleetCategoryWithRelations,
        content: &str,
        embed: &CreateEmbed,
    ) -> Result<(), AppError> {
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = parse_u64_from_string(channel.channel_id.clone())?;
            let channel_id = ChannelId::new(channel_id_u64);

            // Find creation message for this channel if it exists
            let reference_msg = creation_messages
                .iter()
                .filter(|m| m.channel_id == channel_id_u64)
                .max_by_key(|m| &m.created_at);

            let mut message = CreateMessage::new().content(content).embed(embed.clone());

            // If reference message exists, reply to it
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
                            message_type: "reminder".to_string(),
                        })
                        .await?;

                    tracing::info!(
                        "Posted fleet reminder for fleet {} to channel {}",
                        fleet.id,
                        channel_id_u64
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post fleet reminder to channel {}: {}",
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
