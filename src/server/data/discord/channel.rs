use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::GuildChannel;

pub struct DiscordGuildChannelRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildChannelRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Upserts a single channel
    ///
    /// Creates or updates a channel record in the database. Updates the channel's name and
    /// position if it already exists based on channel_id.
    ///
    /// # Arguments
    /// - `guild_id`: Discord's unique identifier for the guild (u64)
    /// - `channel`: Discord GuildChannel object containing channel data
    ///
    /// # Returns
    /// - `Ok(Model)`: Successfully created or updated channel record
    /// - `Err(DbErr)`: Database error during insert/update
    pub async fn upsert(
        &self,
        guild_id: u64,
        channel: &GuildChannel,
    ) -> Result<entity::discord_guild_channel::Model, DbErr> {
        entity::prelude::DiscordGuildChannel::insert(entity::discord_guild_channel::ActiveModel {
            guild_id: ActiveValue::Set(guild_id.to_string()),
            channel_id: ActiveValue::Set(channel.id.get().to_string()),
            name: ActiveValue::Set(channel.name.clone()),
            position: ActiveValue::Set(channel.position as i32),
        })
        .on_conflict(
            OnConflict::column(entity::discord_guild_channel::Column::ChannelId)
                .update_columns([
                    entity::discord_guild_channel::Column::Name,
                    entity::discord_guild_channel::Column::Position,
                ])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await
    }

    /// Deletes a channel by its Discord channel ID
    ///
    /// Removes a channel record from the database when a channel is deleted from Discord.
    ///
    /// # Arguments
    /// - `channel_id`: Discord's unique identifier for the channel (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Channel deleted successfully
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete(&self, channel_id: u64) -> Result<(), DbErr> {
        entity::prelude::DiscordGuildChannel::delete_many()
            .filter(entity::discord_guild_channel::Column::ChannelId.eq(channel_id.to_string()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Retrieves all channels for a specific guild
    ///
    /// Fetches all channel records belonging to a guild, ordered by position.
    ///
    /// # Arguments
    /// - `guild_id`: Discord's unique identifier for the guild (u64)
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: List of channels in the guild
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Vec<entity::discord_guild_channel::Model>, DbErr> {
        use sea_orm::QueryOrder;

        entity::prelude::DiscordGuildChannel::find()
            .filter(entity::discord_guild_channel::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::discord_guild_channel::Column::Position)
            .all(self.db)
            .await
    }
}
