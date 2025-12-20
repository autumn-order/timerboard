//! Fleet notification posting operations.
//!
//! This module provides methods for posting new fleet notifications to Discord channels.
//! It handles creation messages, reminder messages, formup messages, and the upcoming
//! fleets list that displays all scheduled events in a channel.

use dioxus_logger::tracing;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::{
    all::{
        ChannelId, CreateEmbed, CreateMessage, EditMessage, MessageId, MessageReference, Timestamp,
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

use super::builder::FleetNotificationBuilder;

/// Service struct for posting fleet notifications.
pub struct FleetNotificationPosting<'a> {
    /// Database connection for accessing fleet and notification data
    pub db: &'a DatabaseConnection,
    /// Discord HTTP client for sending messages
    pub http: Arc<Http>,
    /// Base application URL for embedding links in notifications
    pub app_url: String,
}

impl<'a> FleetNotificationPosting<'a> {
    /// Creates a new FleetNotificationPosting instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    /// - `http` - Arc-wrapped Discord HTTP client for API requests
    /// - `app_url` - Base URL of the application for embedding in notifications
    ///
    /// # Returns
    /// - `FleetNotificationPosting` - New posting service instance
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
        let builder = FleetNotificationBuilder::new(self.http.clone());
        let commander_name = builder.get_commander_name(fleet, guild_id).await?;

        // Build embed
        let embed = builder
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
}
