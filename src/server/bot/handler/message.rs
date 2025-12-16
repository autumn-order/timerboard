use dioxus_logger::tracing;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serenity::all::{Context, Message};

/// Handle message creation in a channel
pub async fn handle_message(db: &DatabaseConnection, _ctx: Context, message: Message) {
    // Only track messages in guild channels (not DMs)
    if message.guild_id.is_none() {
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
        Ok(None) => return, // Not tracking this channel
        Err(e) => {
            tracing::error!("Failed to check channel fleet list: {}", e);
            return;
        }
    };

    // Don't track the fleet list message itself
    // We want to track OTHER messages (including bot's fleet notifications) that bury the list
    if message_id == existing.message_id {
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
