//! Fleet notification service for Discord message management.
//!
//! This module provides the `FleetNotificationService` for managing Discord notifications
//! related to fleet events. It orchestrates fleet message posting, updates, and cancellations
//! across configured Discord channels with role pings and embedded fleet information. The
//! service also maintains an "upcoming fleets" list message that provides a centralized view
//! of scheduled events in each channel.

use dioxus_logger::tracing;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::{
    all::{
        ChannelId, CreateEmbed, CreateMessage, EditMessage, GuildId, MessageId, MessageReference,
        Timestamp,
    },
    http::Http,
};
use std::sync::Arc;

use crate::server::{
    data::{
        category::FleetCategoryRepository, channel_fleet_list::ChannelFleetListRepository,
        fleet_message::FleetMessageRepository,
    },
    error::AppError,
    model::{
        channel_fleet_list::UpsertChannelFleetListParam,
        fleet::Fleet,
        fleet_message::{CreateFleetMessageParam, FleetMessage},
    },
};

/// Service providing Discord notification operations for fleet events.
///
/// This struct holds references to the database connection, Discord HTTP client, and
/// application URL. It provides methods for posting fleet notifications (creation,
/// reminders, formup), updating existing messages, cancelling fleets, and maintaining
/// an upcoming fleets list in configured channels.
pub struct FleetNotificationService<'a> {
    /// Database connection for accessing fleet and notification data
    db: &'a DatabaseConnection,
    /// Discord HTTP client for sending and editing messages
    http: Arc<Http>,
    /// Base application URL for embedding links in notifications
    app_url: String,
}

impl<'a> FleetNotificationService<'a> {
    /// Creates a new FleetNotificationService instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    /// - `http` - Arc-wrapped Discord HTTP client for API requests
    /// - `app_url` - Base URL of the application for embedding in notifications
    ///
    /// # Returns
    /// - `FleetNotificationService` - New service instance
    pub fn new(db: &'a DatabaseConnection, http: Arc<Http>, app_url: String) -> Self {
        Self { db, http, app_url }
    }

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
        self.post_fleet_notification(
            fleet,
            field_values,
            None, // Title will be built using category name
            0x3498db,
            "creation",
            None,
        )
        .await
    }

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
            return Ok(());
        }

        let message_repo = FleetMessageRepository::new(self.db);
        let creation_messages = message_repo.get_by_fleet_id(fleet.id).await?;

        self.post_fleet_notification(
            fleet,
            field_values,
            None, // Title will be built using category name
            0xf39c12,
            if creation_messages.is_empty() {
                "creation"
            } else {
                "reminder"
            },
            Some(creation_messages),
        )
        .await
    }

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
        let existing_messages = message_repo.get_by_fleet_id(fleet.id).await?;

        self.post_fleet_notification(
            fleet,
            field_values,
            None, // Title will be built using category name
            0xe74c3c,
            "formup",
            Some(existing_messages),
        )
        .await
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
        let category_repo = FleetCategoryRepository::new(self.db);

        // Get all existing messages for this fleet
        let messages = message_repo.get_by_fleet_id(fleet.id).await?;

        if messages.is_empty() {
            return Ok(());
        }

        // Get category data
        let category_data = category_repo
            .get_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        // Get guild_id for fetching commander name
        let guild_id = category_data
            .category
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid guild_id: {}", e)))?;

        // Get ping format fields
        let ping_format = category_data
            .ping_format
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
            .all(self.db)
            .await?;

        // Fetch commander name from Discord
        let commander_name = self.get_commander_name(fleet, guild_id).await?;

        // Build updated embed (use blue color for updates)
        let embed = self
            .build_fleet_embed(
                fleet,
                &fields,
                field_values,
                0x3498db,
                &commander_name,
                &self.app_url,
            )
            .await?;

        // Update each message
        for message in messages {
            let channel_id = message
                .channel_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;
            let msg_id = message
                .message_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid message ID: {}", e)))?;

            let channel_id = ChannelId::new(channel_id);
            let msg_id = MessageId::new(msg_id);

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
            .get_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        let guild_id = category_data
            .category
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid guild_id: {}", e)))?;

        // Fetch commander name from Discord
        let commander_name = self.get_commander_name(fleet, guild_id).await?;

        let commander_id = fleet
            .commander_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid commander ID: {}", e)))?;

        // Build cancellation embed
        let now = chrono::Utc::now();
        let timestamp = Timestamp::from_unix_timestamp(now.timestamp())
            .map_err(|e| AppError::InternalError(format!("Invalid timestamp: {}", e)))?;

        let embed = CreateEmbed::new()
            .title(format!(".:{}  Cancelled:.", category_data.category.name))
            .color(0x95a5a6) // Gray color for cancellation
            .description(format!(
                "{} posted by <@{}>, **{}**, scheduled for **{} UTC** (<t:{}:F>) was cancelled.",
                category_data.category.name,
                commander_id,
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
            let channel_id = message
                .channel_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;
            let msg_id = message
                .message_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid message ID: {}", e)))?;

            let channel_id = ChannelId::new(channel_id);
            let msg_id = MessageId::new(msg_id);

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

    /// Posts or updates the upcoming fleets list for a channel.
    ///
    /// Creates or updates a single message displaying all upcoming non-hidden fleets
    /// for categories configured to post in the channel. The list includes links to
    /// fleet messages and relative timestamps. Intelligently edits the existing list
    /// if it's still the most recent message, or deletes and reposts if other messages
    /// have been sent since. Uses Discord blurple color (0x5865F2).
    ///
    /// # Arguments
    /// - `channel_id_str` - Discord channel ID as string
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted/updated the upcoming fleets list
    /// - `Err(AppError::InternalError)` - Invalid channel or message ID format
    /// - `Err(AppError::Database)` - Database error retrieving fleets or categories
    pub async fn post_upcoming_fleets_list(&self, channel_id_str: &str) -> Result<(), AppError> {
        let channel_id_u64 = channel_id_str
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;

        let channel_id = ChannelId::new(channel_id_u64);
        let now = chrono::Utc::now();

        // Get all categories that post to this channel
        let categories = entity::prelude::FleetCategoryChannel::find()
            .filter(entity::fleet_category_channel::Column::ChannelId.eq(channel_id_str))
            .all(self.db)
            .await?;

        let category_ids: Vec<i32> = categories.iter().map(|c| c.fleet_category_id).collect();

        if category_ids.is_empty() {
            return Ok(());
        }

        // Get all upcoming fleets for these categories
        let fleets = entity::prelude::Fleet::find()
            .filter(entity::fleet::Column::CategoryId.is_in(category_ids.clone()))
            .filter(entity::fleet::Column::FleetTime.gt(now))
            .filter(entity::fleet::Column::Hidden.eq(false))
            .all(self.db)
            .await?;

        if fleets.is_empty() {
            // No upcoming fleets, optionally delete existing list message
            return Ok(());
        }

        // Get categories data
        let categories_data = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?;

        // Get guild_id from the first category for building message links
        let guild_id_str = if let Some(first_category) = categories_data.first() {
            first_category.guild_id.clone()
        } else {
            return Ok(());
        };

        let category_map: std::collections::HashMap<i32, String> = categories_data
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect();

        // Sort fleets by time
        let mut sorted_fleets = fleets;
        sorted_fleets.sort_by_key(|f| f.fleet_time);

        // Build description with bullet list of fleets
        let mut description = String::new();

        for fleet in sorted_fleets {
            let category_name = category_map
                .get(&fleet.category_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            // Get the most recent message for this fleet (prefer reminder over creation)
            let messages = entity::prelude::FleetMessage::find()
                .filter(entity::fleet_message::Column::FleetId.eq(fleet.id))
                .filter(entity::fleet_message::Column::ChannelId.eq(channel_id_str))
                .all(self.db)
                .await?;

            // Find reminder or creation message (not formup)
            let message_link = messages
                .iter()
                .filter(|m| m.message_type == "reminder" || m.message_type == "creation")
                .max_by_key(|m| &m.created_at)
                .map(|m| {
                    format!(
                        "https://discord.com/channels/{}/{}/{}",
                        guild_id_str, channel_id_str, m.message_id
                    )
                });

            if let Some(link) = message_link {
                // Format: • [Fleet Name](link) • Category • relative time
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
        let list_repo = ChannelFleetListRepository::new(self.db);
        let existing_list = list_repo.get_by_channel_id(channel_id_str).await?;

        // Check if we should edit or post new message
        if let Some(existing) = existing_list {
            let msg_id = existing
                .message_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid message ID: {}", e)))?;

            // Compare updated_at (when we posted the list) with last_message_at (most recent message in channel)
            // If our list message is still the most recent, edit it. Otherwise, delete and repost.
            let should_edit = existing.updated_at >= existing.last_message_at;

            tracing::debug!(
                "Channel {}: updated_at={}, last_message_at={}, should_edit={}",
                channel_id_str,
                existing.updated_at,
                existing.last_message_at,
                should_edit
            );

            if should_edit {
                // Edit the existing message since it's still the most recent
                let edit_message = EditMessage::new().embed(embed);

                match self
                    .http
                    .edit_message(channel_id, MessageId::new(msg_id), &edit_message, vec![])
                    .await
                {
                    Ok(_) => {
                        // Update the updated_at timestamp
                        list_repo
                            .upsert(UpsertChannelFleetListParam {
                                channel_id: channel_id_str.to_string(),
                                message_id: msg_id.to_string(),
                            })
                            .await?;
                        tracing::info!(
                            "Edited existing upcoming fleets list in channel {}",
                            channel_id_str
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to edit upcoming fleets list in channel {}: {}",
                            channel_id_str,
                            e
                        );
                    }
                }
            } else {
                // Delete old message and post new one (to be most recent in channel)
                match self
                    .http
                    .delete_message(channel_id, MessageId::new(msg_id), None)
                    .await
                {
                    Ok(_) => {
                        tracing::info!(
                            "Deleted old upcoming fleets list in channel {} (not most recent)",
                            channel_id_str
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to delete old upcoming fleets list in channel {}: {}",
                            channel_id_str,
                            e
                        );
                        // Continue anyway to post new message
                    }
                }

                // Post new message
                let new_message = CreateMessage::new().embed(embed);

                match channel_id.send_message(&self.http, new_message).await {
                    Ok(msg) => {
                        list_repo
                            .upsert(UpsertChannelFleetListParam {
                                channel_id: channel_id_str.to_string(),
                                message_id: msg.id.to_string(),
                            })
                            .await?;
                        tracing::info!(
                            "Posted new upcoming fleets list in channel {}",
                            channel_id_str
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to post upcoming fleets list in channel {}: {}",
                            channel_id_str,
                            e
                        );
                    }
                }
            }
        } else {
            // No existing list, post new message
            let new_message = CreateMessage::new().embed(embed);

            match channel_id.send_message(&self.http, new_message).await {
                Ok(msg) => {
                    list_repo
                        .upsert(UpsertChannelFleetListParam {
                            channel_id: channel_id_str.to_string(),
                            message_id: msg.id.to_string(),
                        })
                        .await?;
                    tracing::info!(
                        "Posted new upcoming fleets list in channel {}",
                        channel_id_str
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post upcoming fleets list in channel {}: {}",
                        channel_id_str,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Core notification posting logic shared by creation, reminder, and formup methods.
    ///
    /// Builds Discord messages with role pings and fleet embeds, then posts them to all
    /// configured channels for the fleet's category. If reference messages are provided,
    /// replies to the most recent message in each channel. Stores posted message IDs in
    /// the database for future updates or cancellations.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    /// - `_title` - Deprecated parameter (title is now built from category name and message type)
    /// - `color` - Embed color as hex integer (e.g., 0x3498db for blue)
    /// - `message_type` - Type identifier for database ("creation", "reminder", "formup")
    /// - `reference_messages` - Optional existing messages to reply to
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted notifications to all channels
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error storing message records
    async fn post_fleet_notification(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
        _title: Option<&str>, // Deprecated - title is now built from category name and message type
        color: u32,
        message_type: &str,
        reference_messages: Option<Vec<FleetMessage>>,
    ) -> Result<(), AppError> {
        // Don't post if fleet is hidden (for creation messages)
        if message_type == "creation" && fleet.hidden {
            return Ok(());
        }

        let category_repo = FleetCategoryRepository::new(self.db);
        let message_repo = FleetMessageRepository::new(self.db);

        // Get category with channels and ping roles
        let category_data = category_repo
            .get_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        // Get guild_id for fetching commander name
        let guild_id = category_data
            .category
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid guild_id: {}", e)))?;

        // Get ping format fields for the category
        let ping_format = category_data
            .ping_format
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
            .all(self.db)
            .await?;

        // Fetch commander name from Discord
        let commander_name = self.get_commander_name(fleet, guild_id).await?;

        // Build embed
        let embed = self
            .build_fleet_embed(
                fleet,
                &fields,
                field_values,
                color,
                &commander_name,
                &self.app_url,
            )
            .await?;

        // Build title based on message type and category name
        let title = match message_type {
            "creation" => format!("**.:New Upcoming {}:.**", category_data.category.name),
            "reminder" => format!(
                "**.:Reminder - Upcoming {}:.**",
                category_data.category.name
            ),
            "formup" => format!("**.:{} Forming Now:.**", category_data.category.name),
            _ => format!("**.:{} Notification:.**", category_data.category.name),
        };

        // Build ping content with title
        let mut content = format!("{}\n\n", title);
        for (ping_role, _) in &category_data.ping_roles {
            let role_id = ping_role
                .role_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid role ID: {}", e)))?;

            // @everyone role has the same ID as the guild - use @everyone instead of <@&guild_id>
            if role_id == guild_id {
                content.push_str("@everyone ");
            } else {
                content.push_str(&format!("<@&{}> ", role_id));
            }
        }

        // Discord doesn't separate space between embed as expected with "\n\n"
        // So we use "\n** **" to newline an invisible character
        content.push_str("\n** **");

        // Post to all configured channels
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = channel
                .channel_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;

            let channel_id = ChannelId::new(channel_id_u64);

            // Find reference message for this channel if it exists
            let reference_msg = reference_messages.as_ref().and_then(|messages| {
                messages
                    .iter()
                    .filter(|m| m.channel_id == channel_id_u64.to_string())
                    .max_by_key(|m| &m.created_at)
            });

            let mut message = CreateMessage::new().content(&content).embed(embed.clone());

            // If reference message exists, reply to it
            if let Some(ref_msg) = reference_msg {
                let msg_id = ref_msg
                    .message_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid message ID: {}", e)))?;
                message = message.reference_message(MessageReference::from((
                    channel_id,
                    MessageId::new(msg_id),
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
                            message_type: message_type.to_string(),
                        })
                        .await?;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post fleet {} to channel {}: {}",
                        message_type,
                        channel_id_u64,
                        e
                    );
                    // Continue posting to other channels even if one fails
                }
            }
        }

        Ok(())
    }

    /// Fetches the commander's Discord name from the guild.
    ///
    /// Attempts to retrieve the fleet commander's display name from the Discord guild.
    /// Prefers the guild nickname if set, otherwise falls back to the Discord username.
    /// If the member cannot be fetched (e.g., they left the guild), returns a fallback
    /// string with their user ID.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing commander_id
    /// - `guild_id` - Discord guild ID as u64 for member lookup
    ///
    /// # Returns
    /// - `Ok(String)` - Commander's nickname, username, or "User {id}" fallback
    /// - `Err(AppError::InternalError)` - Invalid commander ID format
    async fn get_commander_name(&self, fleet: &Fleet, guild_id: u64) -> Result<String, AppError> {
        let commander_id = fleet
            .commander_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid commander ID: {}", e)))?;

        let guild_id = GuildId::new(guild_id);

        // Try to fetch member from guild to get nickname
        match self.http.get_member(guild_id, commander_id.into()).await {
            Ok(member) => {
                // Use nickname if available, otherwise use Discord username
                Ok(member.nick.unwrap_or_else(|| member.user.name.clone()))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch commander {} from guild {}: {}",
                    commander_id,
                    guild_id,
                    e
                );
                // Fallback to just the ID
                Ok(format!("User {}", commander_id))
            }
        }
    }

    /// Builds a Discord embed for a fleet notification.
    ///
    /// Creates a rich embed with fleet details including FC mention, fleet time in both
    /// UTC and local formats, custom ping format fields, and optional description. The
    /// embed includes the fleet name as title, application URL as clickable link, and
    /// a footer with the commander's name and current timestamp.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `fields` - Ping format field definitions from the database
    /// - `field_values` - Map of field_id to value for custom fields
    /// - `color` - Embed color as hex integer
    /// - `commander_name` - Display name of the fleet commander
    /// - `app_url` - Base application URL for embed link
    ///
    /// # Returns
    /// - `Ok(CreateEmbed)` - Discord embed ready for posting
    /// - `Err(AppError::InternalError)` - Invalid commander ID or timestamp format
    async fn build_fleet_embed(
        &self,
        fleet: &Fleet,
        fields: &[entity::ping_format_field::Model],
        field_values: &std::collections::HashMap<i32, String>,
        color: u32,
        commander_name: &str,
        app_url: &str,
    ) -> Result<CreateEmbed, AppError> {
        let commander_id = fleet
            .commander_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid commander ID: {}", e)))?;

        let mut embed = CreateEmbed::new()
            .title(&fleet.name)
            .url(app_url)
            .color(color)
            .field("FC", format!("<@{}>", commander_id), false);

        // Use current time for "sent at" timestamp
        let now = chrono::Utc::now();
        let timestamp = Timestamp::from_unix_timestamp(now.timestamp())
            .map_err(|e| AppError::InternalError(format!("Invalid timestamp: {}", e)))?;

        embed = embed
            .field(
                "Start Time (UTC)",
                format!(
                    "{} EVE Time",
                    fleet.fleet_time.format("%Y-%m-%d %H:%M").to_string()
                ),
                false,
            )
            .field(
                "Start Time (Local)",
                format!(
                    "<t:{}:F> - <t:{}:R>",
                    fleet.fleet_time.timestamp(),
                    fleet.fleet_time.timestamp()
                ),
                false,
            );

        // Add custom fields from ping format
        for field in fields {
            if let Some(value) = field_values.get(&field.id) {
                if !value.is_empty() {
                    embed = embed.field(&field.name, value, false);
                }
            }
        }

        // Add description if present
        if let Some(description) = &fleet.description {
            if !description.is_empty() {
                embed = embed.field("Additional Information", description, false);
            }
        }

        // Footer with commander name
        embed = embed.footer(serenity::all::CreateEmbedFooter::new(format!(
            "Sent by: {}",
            commander_name
        )));

        embed = embed.timestamp(timestamp);

        Ok(embed)
    }
}
