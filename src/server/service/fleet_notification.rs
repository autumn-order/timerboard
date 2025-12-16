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
    data::{category::FleetCategoryRepository, fleet_message::FleetMessageRepository},
    error::AppError,
};

pub struct FleetNotificationService<'a> {
    db: &'a DatabaseConnection,
    http: Arc<Http>,
    app_url: String,
}

impl<'a> FleetNotificationService<'a> {
    pub fn new(db: &'a DatabaseConnection, http: Arc<Http>, app_url: String) -> Self {
        Self { db, http, app_url }
    }

    /// Posts fleet creation message to all configured channels
    ///
    /// Only posts if fleet is not hidden. Stores message IDs in database.
    ///
    /// # Arguments
    /// - `fleet`: Fleet entity model
    /// - `field_values`: Map of field_id -> value for ping format fields
    pub async fn post_fleet_creation(
        &self,
        fleet: &entity::fleet::Model,
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

    /// Posts fleet reminder message as a reply to the creation message
    ///
    /// If no creation message exists (fleet was hidden), posts as a new message.
    ///
    /// # Arguments
    /// - `fleet`: Fleet entity model
    /// - `field_values`: Map of field_id -> value for ping format fields
    pub async fn post_fleet_reminder(
        &self,
        fleet: &entity::fleet::Model,
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

    /// Posts fleet form-up message as a reply to the creation/reminder message
    ///
    /// # Arguments
    /// - `fleet`: Fleet entity model
    /// - `field_values`: Map of field_id -> value for ping format fields
    pub async fn post_fleet_formup(
        &self,
        fleet: &entity::fleet::Model,
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

    /// Updates all existing fleet messages with new fleet information
    ///
    /// # Arguments
    /// - `fleet`: Updated fleet entity model
    /// - `field_values`: Map of field_id -> value for ping format fields
    pub async fn update_fleet_messages(
        &self,
        fleet: &entity::fleet::Model,
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

    /// Cancels all existing fleet messages by editing them with cancellation notice
    ///
    /// # Arguments
    /// - `fleet`: Fleet entity model being cancelled
    pub async fn cancel_fleet_messages(
        &self,
        fleet: &entity::fleet::Model,
    ) -> Result<(), AppError> {
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

    /// Core notification posting logic
    ///
    /// # Arguments
    /// - `fleet`: Fleet entity model
    /// - `field_values`: Map of field_id -> value for ping format fields
    /// - `title`: Message title to display above the embed
    /// - `color`: Embed color
    /// - `message_type`: Type of message for database storage
    /// - `reference_messages`: Optional existing messages to reply to
    async fn post_fleet_notification(
        &self,
        fleet: &entity::fleet::Model,
        field_values: &std::collections::HashMap<i32, String>,
        _title: Option<&str>, // Deprecated - title is now built from category name and message type
        color: u32,
        message_type: &str,
        reference_messages: Option<Vec<entity::fleet_message::Model>>,
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
            "reminder" => format!("**.:Reminder: Upcoming {}:.**", category_data.category.name),
            "formup" => format!("**.:{}  Forming Now!:.**", category_data.category.name),
            _ => format!("**.:{}  Notification:.**", category_data.category.name),
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
                        .create(fleet.id, channel_id_u64, msg.id.get(), message_type)
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

    /// Fetches the commander's Discord name (nickname in guild or username fallback)
    async fn get_commander_name(
        &self,
        fleet: &entity::fleet::Model,
        guild_id: u64,
    ) -> Result<String, AppError> {
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

    /// Builds a Discord embed for a fleet
    async fn build_fleet_embed(
        &self,
        fleet: &entity::fleet::Model,
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
