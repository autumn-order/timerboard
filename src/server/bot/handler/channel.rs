//! Channel event handlers for Discord guild channels.
//!
//! This module handles Discord events related to guild channels, specifically text
//! channels that can be used for fleet notifications. The handlers keep the database
//! synchronized with Discord's channel state to enable:
//! - Channel selection in notification configuration UI
//! - Validation that configured channels still exist
//! - Proper cleanup when channels are deleted
//!
//! Only text channels are tracked, as they are the only channel type that can receive
//! fleet notification messages and embeds.

use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{ChannelType, Context, GuildChannel, Message};

use crate::server::data::discord::DiscordGuildChannelRepository;

/// Handles the channel_create event when a channel is created in a guild.
///
/// Adds the channel to the database if it's a text channel. This makes the channel
/// available for selection in the fleet notification configuration UI. Non-text
/// channels (voice, announcement, etc.) are ignored as they cannot receive messages.
///
/// # Arguments
/// - `db` - Database connection for creating the channel record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `channel` - The newly created guild channel from Discord
pub async fn handle_channel_create(db: &DatabaseConnection, _ctx: Context, channel: GuildChannel) {
    let guild_id = channel.guild_id.get();
    let channel_repo = DiscordGuildChannelRepository::new(db);

    // Only track text channels
    if channel.kind != ChannelType::Text {
        tracing::debug!(
            "Ignoring non-text channel {} (type: {:?}) in guild {}",
            channel.name,
            channel.kind,
            guild_id
        );
        return;
    }

    if let Err(e) = channel_repo.upsert(guild_id, &channel).await {
        tracing::error!(
            "Failed to upsert new channel {} in guild {}: {:?}",
            channel.name,
            guild_id,
            e
        );
    } else {
        tracing::debug!("Created channel {} in guild {}", channel.name, guild_id);
    }
}

/// Handles the channel_update event when a channel is updated in a guild.
///
/// Updates the channel's information (name, permissions, etc.) in the database if
/// it's a text channel. This ensures the UI displays current channel names and that
/// notification configurations remain valid. Non-text channels are ignored.
///
/// # Arguments
/// - `db` - Database connection for updating the channel record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `_old` - Previous channel state if available (unused)
/// - `new` - Updated channel state from Discord
pub async fn handle_channel_update(
    db: &DatabaseConnection,
    _ctx: Context,
    _old: Option<GuildChannel>,
    new: GuildChannel,
) {
    let channel = new;
    let guild_id = channel.guild_id.get();
    let channel_repo = DiscordGuildChannelRepository::new(db);

    // Only track text channels
    if channel.kind != ChannelType::Text {
        tracing::debug!(
            "Ignoring non-text channel {} (type: {:?}) in guild {}",
            channel.name,
            channel.kind,
            guild_id
        );
        return;
    }

    if let Err(e) = channel_repo.upsert(guild_id, &channel).await {
        tracing::error!(
            "Failed to upsert updated channel {} in guild {}: {:?}",
            channel.name,
            guild_id,
            e
        );
    } else {
        tracing::debug!("Updated channel {} in guild {}", channel.name, guild_id);
    }
}

/// Handles the channel_delete event when a channel is deleted from a guild.
///
/// Removes the channel from the database. Any fleet notification configurations
/// that reference this channel are automatically cleaned up via database CASCADE
/// constraints, preventing orphaned configuration entries.
///
/// # Arguments
/// - `db` - Database connection for deleting the channel record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `channel` - The deleted guild channel from Discord
/// - `_messages` - Messages that were in the channel if available (unused)
pub async fn handle_channel_delete(
    db: &DatabaseConnection,
    _ctx: Context,
    channel: GuildChannel,
    _messages: Option<Vec<Message>>,
) {
    let guild_id = channel.guild_id.get();
    let channel_id = channel.id.get();
    let channel_repo = DiscordGuildChannelRepository::new(db);

    if let Err(e) = channel_repo.delete(channel_id).await {
        tracing::error!(
            "Failed to delete channel {} from guild {}: {:?}",
            channel_id,
            guild_id,
            e
        );
    } else {
        tracing::debug!("Deleted channel {} from guild {}", channel_id, guild_id);
    }
}
