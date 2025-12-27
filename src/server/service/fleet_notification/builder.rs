//! Fleet notification embed builder utilities.
//!
//! This module provides helper functions for constructing Discord embeds and fetching
//! display names for fleet notifications. These utilities are shared across posting
//! and update operations to ensure consistent formatting.

use dioxus_logger::tracing;
use serenity::{
    all::{CreateEmbed, GuildId, Timestamp},
    http::Http,
};
use std::sync::Arc;

use crate::{
    model::ping_format::PingFormatFieldType,
    server::{
        error::{internal::InternalError, AppError},
        model::{fleet::Fleet, ping_format::PingFormatField},
    },
};

/// Fetches the commander's Discord name from the guild.
///
/// Attempts to retrieve the fleet commander's display name from the Discord guild.
/// Prefers the guild nickname if set, otherwise falls back to the Discord username.
/// If the member cannot be fetched (e.g., they left the guild), returns a fallback
/// string with their user ID.
///
/// # Arguments
/// - `http` - Discord HTTP client for API requests
/// - `fleet` - Fleet domain model containing commander_id
/// - `guild_id` - Discord guild ID as u64 for member lookup
///
/// # Returns
/// - `Ok(String)` - Commander's nickname, username, or "User {id}" fallback
/// - `Err(AppError::InternalError)` - Invalid commander ID format
pub async fn get_commander_name(
    http: Arc<Http>,
    fleet: &Fleet,
    guild_id: u64,
) -> Result<String, AppError> {
    let guild_id = GuildId::new(guild_id);

    // Try to fetch member from guild to get nickname
    match http.get_member(guild_id, fleet.commander_id.into()).await {
        Ok(member) => {
            // Use nickname if available, otherwise use Discord username
            Ok(member.nick.unwrap_or_else(|| member.user.name.clone()))
        }
        Err(e) => {
            tracing::warn!(
                "Failed to fetch commander {} from guild {}: {}",
                fleet.commander_id,
                guild_id,
                e
            );
            // Fallback to just the ID
            Ok(format!("User {}", fleet.commander_id))
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
pub async fn build_fleet_embed(
    fleet: &Fleet,
    fields: &[PingFormatField],
    field_values: &std::collections::HashMap<i32, String>,
    color: u32,
    commander_name: &str,
    app_url: &str,
) -> Result<CreateEmbed, AppError> {
    let mut embed = CreateEmbed::new()
        .title(&fleet.name)
        .url(app_url)
        .color(color)
        .field("FC", format!("<@{}>", fleet.commander_id), false);

    // Use current time for "sent at" timestamp
    let now = chrono::Utc::now();
    let timestamp = Timestamp::from_unix_timestamp(now.timestamp()).map_err(|e| {
        AppError::InternalError(InternalError::InvalidDiscordTimestamp {
            timestamp: now.timestamp(),
            reason: e.to_string(),
        })
    })?;

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
                // Format boolean values as "Yes"/"No" for better readability
                let display_value = match field.field_type {
                    PingFormatFieldType::Bool => match value.as_str() {
                        "true" => "Yes",
                        "false" => "No",
                        _ => value.as_str(),
                    },
                    PingFormatFieldType::Text => value.as_str(),
                };
                embed = embed.field(&field.name, display_value, false);
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
