//! Message event handlers for tracking fleet list visibility.
//!
//! This module handles Discord message events to track when fleet list messages
//! are "buried" by other messages in a channel. The fleet list is a pinned message
//! that displays upcoming fleets, and it needs to be reposted when it gets pushed
//! too far up in the channel by newer messages.
//!
//! The handler tracks the timestamp of the last message in channels that have
//! fleet lists, updating a `last_message_at` field. A separate process (likely
//! a scheduler or manual trigger) uses this timestamp to determine when the fleet
//! list needs to be reposted to keep it visible.
//!
//! Only messages in guild channels are tracked (DMs are ignored), and the fleet
//! list message itself is not counted to avoid false positives.

use dioxus_logger::tracing;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serenity::all::{Context, Message};

/// Handles message creation in a channel.
///
/// Tracks messages in channels that have fleet list messages configured. When a
/// message is posted in a tracked channel, updates the `last_message_at` timestamp
/// to indicate that new content has been posted after the fleet list.
///
/// This timestamp is used by other parts of the system to determine when the fleet
/// list has been "buried" by enough messages that it should be reposted to remain
/// visible to users.
///
/// Messages in the following scenarios are ignored:
/// - Direct messages (not in a guild)
/// - Messages in channels without a fleet list configured
/// - The fleet list message itself (to avoid false positives)
///
/// Note: Only `last_message_at` is updated. The `updated_at` field is reserved for
/// when the bot actually posts or edits the fleet list message itself.
///
/// # Arguments
/// - `db` - Database connection for querying and updating channel fleet list records
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `message` - The message that was posted from Discord
pub async fn handle_message(db: &DatabaseConnection, _ctx: Context, message: Message) {
    // Only track messages in guild channels (not DMs)
    if message.guild_id.is_none() {
        tracing::debug!(
            "Ignoring DM message {} from user {}",
            message.id,
            message.author.name
        );
        return;
    }

    let channel_id = message.channel_id.to_string();
    let message_id = message.id.to_string();
    let message_timestamp = message.timestamp.to_utc();

    // Check if this channel has a fleet list we're tracking
    let existing = match entity::prelude::ChannelFleetList::find()
        .filter(entity::channel_fleet_list::Column::ChannelId.eq(&channel_id))
        .one(db)
        .await
    {
        Ok(Some(record)) => record,
        Ok(None) => {
            // Not tracking this channel
            tracing::debug!(
                "Channel {} does not have a fleet list, ignoring message",
                channel_id
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                "Failed to check channel fleet list for channel {}: {}",
                channel_id,
                e
            );
            return;
        }
    };

    // Don't track the fleet list message itself
    // We want to track OTHER messages (including bot's fleet notifications) that bury the list
    if message_id == existing.message_id {
        tracing::debug!(
            "Ignoring fleet list message {} itself in channel {}",
            message_id,
            channel_id
        );
        return;
    }

    // Update the last_message_at timestamp (but NOT updated_at)
    // updated_at should only be changed when the bot posts/edits the fleet list
    let mut active: entity::channel_fleet_list::ActiveModel = existing.into();
    active.last_message_at = ActiveValue::Set(message_timestamp);

    if let Err(e) = active.update(db).await {
        tracing::error!(
            "Failed to update last_message_at for channel {}: {}",
            channel_id,
            e
        );
    } else {
        tracing::debug!(
            "Updated last_message_at for channel {} to {}",
            channel_id,
            message_timestamp
        );
    }
}
