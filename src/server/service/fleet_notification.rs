use chrono::{DateTime, Utc};
use dioxus_logger::tracing;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::{
    all::{
        ChannelId, CreateEmbed, CreateMessage, EditMessage, MessageId, MessageReference, Timestamp,
        UserId,
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
}

impl<'a> FleetNotificationService<'a> {
    pub fn new(db: &'a DatabaseConnection, http: Arc<Http>) -> Self {
        Self { db, http }
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
        // Don't post if fleet is hidden
        if fleet.hidden {
            return Ok(());
        }

        let category_repo = FleetCategoryRepository::new(self.db);
        let message_repo = FleetMessageRepository::new(self.db);

        // Get category with channels and ping roles
        let category_data = category_repo
            .get_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        // Get ping format fields for the category
        let ping_format = category_data
            .ping_format
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
            .all(self.db)
            .await?;

        // Build embed
        let embed = self.build_fleet_embed(
            "**.:New Upcoming Fleet:.**",
            fleet,
            &fields,
            field_values,
            0x3498db, // Blue color
        )?;

        // Build ping content
        let mut ping_content = String::new();
        for (ping_role, _) in &category_data.ping_roles {
            let role_id = ping_role
                .role_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid role ID: {}", e)))?;
            ping_content.push_str(&format!("<@&{}> ", role_id));
        }

        // Post to all configured channels
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = channel
                .channel_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;

            let channel_id = ChannelId::new(channel_id_u64);

            // Create and send message
            let message = CreateMessage::new()
                .content(&ping_content)
                .embed(embed.clone());

            match channel_id.send_message(&self.http, message).await {
                Ok(msg) => {
                    // Store message in database
                    message_repo
                        .create(fleet.id, channel_id_u64, msg.id.get(), "creation")
                        .await?;
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

        let category_repo = FleetCategoryRepository::new(self.db);
        let message_repo = FleetMessageRepository::new(self.db);

        // Get category with channels and ping roles
        let category_data = category_repo
            .get_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        // Get ping format fields
        let ping_format = category_data
            .ping_format
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
            .all(self.db)
            .await?;

        // Build embed
        let embed = self.build_fleet_embed(
            "**.:Reminder: Upcoming Fleet:.**",
            fleet,
            &fields,
            field_values,
            0xf39c12, // Orange color
        )?;

        // Build ping content
        let mut ping_content = String::new();
        for (ping_role, _) in &category_data.ping_roles {
            let role_id = ping_role
                .role_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid role ID: {}", e)))?;
            ping_content.push_str(&format!("<@&{}> ", role_id));
        }

        // Get existing creation messages
        let creation_messages = message_repo.get_by_fleet_id(fleet.id).await?;
        let has_creation = !creation_messages.is_empty();

        // Post to all configured channels
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = channel
                .channel_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;

            let channel_id = ChannelId::new(channel_id_u64);

            // Find creation message for this channel if it exists
            let creation_msg = creation_messages
                .iter()
                .find(|m| m.channel_id == channel_id_u64.to_string());

            let message = CreateMessage::new()
                .content(&ping_content)
                .embed(embed.clone());

            // If creation message exists, reply to it; otherwise post new message
            let message = if let Some(creation) = creation_msg {
                let msg_id = creation
                    .message_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid message ID: {}", e)))?;
                message
                    .reference_message(MessageReference::from((channel_id, MessageId::new(msg_id))))
            } else {
                message
            };

            match channel_id.send_message(&self.http, message).await {
                Ok(msg) => {
                    // Store message in database
                    let msg_type = if has_creation { "reminder" } else { "creation" };
                    message_repo
                        .create(fleet.id, channel_id_u64, msg.id.get(), msg_type)
                        .await?;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post fleet reminder to channel {}: {}",
                        channel_id_u64,
                        e
                    );
                }
            }
        }

        Ok(())
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
        let category_repo = FleetCategoryRepository::new(self.db);
        let message_repo = FleetMessageRepository::new(self.db);

        // Get category with channels and ping roles
        let category_data = category_repo
            .get_by_id(fleet.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        // Get ping format fields
        let ping_format = category_data
            .ping_format
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
            .all(self.db)
            .await?;

        // Build embed
        let embed = self.build_fleet_embed(
            "**.:Fleet Forming Now!:.**",
            fleet,
            &fields,
            field_values,
            0xe74c3c, // Red color
        )?;

        // Build ping content
        let mut ping_content = String::new();
        for (ping_role, _) in &category_data.ping_roles {
            let role_id = ping_role
                .role_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid role ID: {}", e)))?;
            ping_content.push_str(&format!("<@&{}> ", role_id));
        }

        // Get existing messages (prefer reminder, fallback to creation)
        let existing_messages = message_repo.get_by_fleet_id(fleet.id).await?;

        // Post to all configured channels
        for (channel, _) in &category_data.channels {
            let channel_id_u64 = channel
                .channel_id
                .parse::<u64>()
                .map_err(|e| AppError::InternalError(format!("Invalid channel ID: {}", e)))?;

            let channel_id = ChannelId::new(channel_id_u64);

            // Find the most recent message for this channel (reminder or creation)
            let reference_msg = existing_messages
                .iter()
                .filter(|m| m.channel_id == channel_id_u64.to_string())
                .max_by_key(|m| &m.created_at);

            let message = CreateMessage::new()
                .content(&ping_content)
                .embed(embed.clone());

            // Reply to existing message if found, otherwise post new
            let message = if let Some(ref_msg) = reference_msg {
                let msg_id = ref_msg
                    .message_id
                    .parse::<u64>()
                    .map_err(|e| AppError::InternalError(format!("Invalid message ID: {}", e)))?;
                message
                    .reference_message(MessageReference::from((channel_id, MessageId::new(msg_id))))
            } else {
                message
            };

            match channel_id.send_message(&self.http, message).await {
                Ok(msg) => {
                    // Store message in database
                    message_repo
                        .create(fleet.id, channel_id_u64, msg.id.get(), "formup")
                        .await?;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to post fleet form-up to channel {}: {}",
                        channel_id_u64,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Builds a Discord embed for a fleet
    fn build_fleet_embed(
        &self,
        title: &str,
        fleet: &entity::fleet::Model,
        fields: &[entity::ping_format_field::Model],
        field_values: &std::collections::HashMap<i32, String>,
        color: u32,
    ) -> Result<CreateEmbed, AppError> {
        let commander_id = fleet
            .commander_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Invalid commander ID: {}", e)))?;

        let mut embed = CreateEmbed::new()
            .title(title)
            .color(color)
            .field("Fleet Name", &fleet.name, false)
            .field("FC", format!("<@{}>", commander_id), false);

        // Format time as Discord timestamp
        let timestamp = Timestamp::from_unix_timestamp(fleet.fleet_time.timestamp())
            .map_err(|e| AppError::InternalError(format!("Invalid timestamp: {}", e)))?;

        embed = embed
            .field(
                "Start Time (UTC)",
                fleet.fleet_time.format("%Y-%m-%d %H:%M").to_string(),
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

        // Footer with commander
        embed = embed.footer(
            serenity::all::CreateEmbedFooter::new(format!("Sent by: {}", commander_id))
                .text(format!("Sent by: <@{}>", commander_id)),
        );

        embed = embed.timestamp(timestamp);

        Ok(embed)
    }
}
