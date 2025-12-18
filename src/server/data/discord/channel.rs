use crate::server::model::discord::DiscordGuildChannel;
use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::GuildChannel;

/// Repository for Discord guild channel database operations.
///
/// Provides CRUD operations for Discord channels, converting between entity models
/// and domain models at the infrastructure boundary. Handles channel creation,
/// updates, deletion, and retrieval operations.
pub struct DiscordGuildChannelRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildChannelRepository<'a> {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    /// - `db` - Database connection reference
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Upserts a single channel from Discord API data.
    ///
    /// Creates or updates a channel record in the database. Updates the channel's name and
    /// position if it already exists based on channel_id. Converts the entity model to a
    /// domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `guild_id` - Discord's unique identifier for the guild
    /// - `channel` - Discord GuildChannel object containing channel data
    ///
    /// # Returns
    /// - `Ok(DiscordGuildChannel)` - Successfully created or updated channel domain model
    /// - `Err(DbErr)` - Database error during insert/update or entity conversion failure
    pub async fn upsert(
        &self,
        guild_id: u64,
        channel: &GuildChannel,
    ) -> Result<DiscordGuildChannel, DbErr> {
        let entity = entity::prelude::DiscordGuildChannel::insert(
            entity::discord_guild_channel::ActiveModel {
                guild_id: ActiveValue::Set(guild_id.to_string()),
                channel_id: ActiveValue::Set(channel.id.get().to_string()),
                name: ActiveValue::Set(channel.name.clone()),
                position: ActiveValue::Set(channel.position as i32),
            },
        )
        .on_conflict(
            OnConflict::column(entity::discord_guild_channel::Column::ChannelId)
                .update_columns([
                    entity::discord_guild_channel::Column::Name,
                    entity::discord_guild_channel::Column::Position,
                ])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await?;

        DiscordGuildChannel::from_entity(entity)
    }

    /// Deletes a channel by its Discord channel ID.
    ///
    /// Removes a channel record from the database when a channel is deleted from Discord.
    /// This operation cascades to related records due to foreign key constraints.
    ///
    /// # Arguments
    /// - `channel_id` - Discord's unique identifier for the channel
    ///
    /// # Returns
    /// - `Ok(())` - Channel deleted successfully (or didn't exist)
    /// - `Err(DbErr)` - Database error during deletion
    pub async fn delete(&self, channel_id: u64) -> Result<(), DbErr> {
        entity::prelude::DiscordGuildChannel::delete_many()
            .filter(entity::discord_guild_channel::Column::ChannelId.eq(channel_id.to_string()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Retrieves all channels for a specific guild.
    ///
    /// Fetches all channel records belonging to a guild, ordered by position for
    /// display purposes. Converts entity models to domain models at the repository boundary.
    ///
    /// # Arguments
    /// - `guild_id` - Discord's unique identifier for the guild
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuildChannel>)` - List of channel domain models in position order
    /// - `Err(DbErr)` - Database error during query or entity conversion failure
    pub async fn get_by_guild_id(&self, guild_id: u64) -> Result<Vec<DiscordGuildChannel>, DbErr> {
        use sea_orm::QueryOrder;

        let entities = entity::prelude::DiscordGuildChannel::find()
            .filter(entity::discord_guild_channel::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::discord_guild_channel::Column::Position)
            .all(self.db)
            .await?;

        entities
            .into_iter()
            .map(DiscordGuildChannel::from_entity)
            .collect()
    }
}
