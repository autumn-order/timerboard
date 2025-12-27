//! Fleet creation notification operations.
//!
//! This module provides functionality for posting the initial creation notifications for new fleets.
//! Creation messages are posted to all configured channels with role pings and fleet details.

use dioxus_logger::tracing;
use serenity::all::{ChannelId, CreateEmbed, CreateMessage};

use crate::server::{
    data::fleet_message::FleetMessageRepository,
    error::AppError,
    model::{
        category::FleetCategoryWithRelations, fleet::Fleet, fleet_message::CreateFleetMessageParam,
    },
    util::parse::parse_u64_from_string,
};

use super::FleetNotificationService;

impl<'a> FleetNotificationService<'a> {
    /// Posts fleet creation message to all configured channels.
    ///
    /// Creates Discord messages with fleet details in all channels configured for the
    /// fleet's category. Only posts if the fleet is not hidden. Message IDs are stored
    /// in the database for later updates or cancellations. Uses blue embed color (0x3498db).
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted creation messages to all channels
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error storing message records
    pub async fn post_fleet_creation(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        // Don't post if fleet is hidden
        if fleet.hidden {
            return Ok(());
        }

        let message_repo = FleetMessageRepository::new(self.db);

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
                0x3498db, // Blue color for creation
                guild_id,
            )
            .await?;

        // Build title based on category name
        let title = format!("**.:New Upcoming {}:.**", category_data.category.name);

        // Build ping content with title
        let content = self.build_ping_content(&title, &category_data, guild_id)?;

        // Post to all configured channels
        self.post_creation_messages(fleet, &message_repo, &category_data, &content, &embed)
            .await
    }

    /// Posts fleet creation messages to all configured channels.
    ///
    /// # Arguments
    /// - `fleet` - Fleet data
    /// - `message_repo` - Fleet message repository
    /// - `category_data` - Category data with channels
    /// - `content` - Message content with role pings
    /// - `embed` - Fleet embed to post
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted to all channels (or failed gracefully)
    /// - `Err(AppError)` - Critical error (database or parsing)
    async fn post_creation_messages(
        &self,
        fleet: &Fleet,
        message_repo: &FleetMessageRepository<'_>,
        category_data: &FleetCategoryWithRelations,
        content: &str,
        embed: &CreateEmbed,
    ) -> Result<(), AppError> {
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = parse_u64_from_string(channel.channel_id.clone())?;
            let channel_id = ChannelId::new(channel_id_u64);

            let message = CreateMessage::new().content(content).embed(embed.clone());

            match channel_id.send_message(&self.http, message).await {
                Ok(msg) => {
                    // Store message in database
                    message_repo
                        .create(CreateFleetMessageParam {
                            fleet_id: fleet.id,
                            channel_id: channel_id_u64,
                            message_id: msg.id.get(),
                            message_type: "creation".to_string(),
                        })
                        .await?;

                    tracing::info!(
                        "Posted fleet creation for fleet {} to channel {}",
                        fleet.id,
                        channel_id_u64
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post fleet creation to channel {}: {}",
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
