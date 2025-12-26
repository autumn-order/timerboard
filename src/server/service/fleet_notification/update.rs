//! Fleet notification update and cancellation operations.
//!
//! This module provides methods for updating existing fleet notifications and cancelling
//! fleets. It handles editing Discord messages with new fleet information and posting
//! cancellation notices when fleets are cancelled.

use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::{
    all::{ChannelId, CreateEmbed, EditMessage, MessageId, Timestamp},
    http::Http,
};
use std::sync::Arc;

use crate::server::{
    data::{
        category::FleetCategoryRepository, fleet_message::FleetMessageRepository,
        ping_format::field::PingFormatFieldRepository,
    },
    error::{internal::InternalError, AppError},
    model::fleet::Fleet,
    util::parse::parse_u64_from_string,
};

use super::builder::FleetNotificationBuilder;

/// Service struct for updating and cancelling fleet notifications.
pub struct FleetNotificationUpdate<'a> {
    /// Database connection for accessing fleet and notification data
    pub db: &'a DatabaseConnection,
    /// Discord HTTP client for editing messages
    pub http: Arc<Http>,
}

impl<'a> FleetNotificationUpdate<'a> {
    /// Creates a new FleetNotificationUpdate instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    /// - `http` - Arc-wrapped Discord HTTP client for API requests
    ///
    /// # Returns
    /// - `FleetNotificationUpdate` - New update service instance
    pub fn new(db: &'a DatabaseConnection, http: Arc<Http>) -> Self {
        Self { db, http }
    }

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
    /// - `app_url` - Base application URL for embed link
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
        app_url: &str,
    ) -> Result<(), AppError> {
        let message_repo = FleetMessageRepository::new(self.db);
        let ping_format_field_repo = PingFormatFieldRepository::new(self.db);
        let category_repo = FleetCategoryRepository::new(self.db);

        // Get all existing messages for this fleet
        let messages = message_repo.get_by_fleet_id(fleet.id).await?;

        if messages.is_empty() {
            return Ok(());
        }

        // Get category data
        let category_data = category_repo
            .find_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        // Get guild_id for fetching commander name
        let guild_id = parse_u64_from_string(category_data.category.guild_id)?;

        // Get ping format fields
        let ping_format = category_data
            .ping_format
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = ping_format_field_repo
            .get_by_ping_format_id(guild_id, ping_format.id)
            .await?;

        // Fetch commander name from Discord
        let builder = FleetNotificationBuilder::new(self.http.clone());
        let commander_name = builder.get_commander_name(fleet, guild_id).await?;

        // Build updated embed (use blue color for updates)
        let embed = builder
            .build_fleet_embed(
                fleet,
                &fields,
                field_values,
                0x3498db,
                &commander_name,
                app_url,
            )
            .await?;

        // Update each message
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
    pub async fn cancel_fleet_messages(&self, fleet: &Fleet) -> Result<(), AppError> {
        let message_repo = FleetMessageRepository::new(self.db);
        let category_repo = FleetCategoryRepository::new(self.db);

        // Get all existing messages for this fleet
        let messages = message_repo.get_by_fleet_id(fleet.id).await?;

        if messages.is_empty() {
            return Ok(());
        }

        // Get category data for guild_id
        let category_data = category_repo
            .find_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        let guild_id = parse_u64_from_string(category_data.category.guild_id)?;

        // Fetch commander name from Discord
        let builder = FleetNotificationBuilder::new(self.http.clone());
        let commander_name = builder.get_commander_name(fleet, guild_id).await?;

        // Build cancellation embed
        let now = chrono::Utc::now();
        let timestamp = Timestamp::from_unix_timestamp(now.timestamp()).map_err(|e| {
            AppError::InternalError(InternalError::InvalidDiscordTimestamp {
                timestamp: now.timestamp(),
                reason: e.to_string(),
            })
        })?;

        let embed = CreateEmbed::new()
            .title(format!(".:{}  Cancelled:.", category_data.category.name))
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
