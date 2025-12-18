//! Discord guild repository for database operations.
//!
//! This module provides the `DiscordGuildRepository` for managing Discord guild
//! (server) records in the database. It handles upserting guilds from Discord,
//! tracking sync timestamps, and querying guilds by user membership. Guild data
//! is synced from Discord via Serenity and stored locally for display and to
//! prevent excessive re-syncing.
//!
//! All methods return domain models at the repository boundary, converting SeaORM
//! entity models internally to prevent database-specific structures from leaking
//! into service and controller layers.

use chrono::{Duration, Utc};
use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::Guild;

use crate::server::model::discord::DiscordGuild;

/// Repository for Discord guild database operations.
///
/// Provides methods for upserting, querying, and managing sync timestamps for
/// Discord guilds. Used to keep local guild data synchronized with Discord's state.
pub struct DiscordGuildRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRepository<'a> {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    /// - `db` - Database connection for executing queries
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Upserts a Discord guild (insert or update if exists).
    ///
    /// Creates a new guild record or updates an existing one based on guild_id.
    /// Updates name and icon_hash if the guild already exists. The last_sync_at
    /// timestamp is not updated by this method - use `update_last_sync()` after
    /// completing a full sync.
    ///
    /// # Arguments
    /// - `guild` - Serenity guild object containing guild data from Discord
    ///
    /// # Returns
    /// - `Ok(DiscordGuild)` - The upserted guild as a domain model
    /// - `Err(DbErr)` - Database error during upsert operation
    pub async fn upsert(&self, guild: Guild) -> Result<DiscordGuild, DbErr> {
        let entity = entity::prelude::DiscordGuild::insert(entity::discord_guild::ActiveModel {
            guild_id: ActiveValue::Set(guild.id.get().to_string()),
            name: ActiveValue::Set(guild.name),
            icon_hash: ActiveValue::Set(guild.icon_hash.map(|i| i.to_string())),
            last_sync_at: ActiveValue::NotSet,
        })
        .on_conflict(
            OnConflict::column(entity::discord_guild::Column::GuildId)
                .update_columns([entity::discord_guild::Column::Name])
                .update_columns([entity::discord_guild::Column::IconHash])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await?;

        DiscordGuild::from_entity(entity)
    }

    /// Gets all guilds in the database.
    ///
    /// Retrieves all guild records, typically used for administrative purposes
    /// or bot-wide operations like syncing all guilds.
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuild>)` - Vector of all guild records
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_all(&self) -> Result<Vec<DiscordGuild>, DbErr> {
        let entities = entity::prelude::DiscordGuild::find().all(self.db).await?;

        entities
            .into_iter()
            .map(DiscordGuild::from_entity)
            .collect()
    }

    /// Finds a guild by its Discord guild ID.
    ///
    /// Searches for a guild in the database using the Discord-assigned guild ID.
    /// Used to check if the bot is present in a specific guild or to retrieve
    /// guild information for display.
    ///
    /// # Arguments
    /// - `guild_id` - Discord's unique identifier for the guild
    ///
    /// # Returns
    /// - `Ok(Some(DiscordGuild))` - Guild found in database
    /// - `Ok(None)` - Guild not found (bot not in this guild)
    /// - `Err(DbErr)` - Database error during query
    pub async fn find_by_guild_id(&self, guild_id: u64) -> Result<Option<DiscordGuild>, DbErr> {
        let entity = entity::prelude::DiscordGuild::find()
            .filter(entity::discord_guild::Column::GuildId.eq(guild_id.to_string()))
            .one(self.db)
            .await?;

        entity.map(DiscordGuild::from_entity).transpose()
    }

    /// Gets all guilds for a specific user.
    ///
    /// Retrieves all guilds that the specified user is a member of by joining
    /// with the guild_member table. Used to determine which timerboards are
    /// available to a user for display in guild selection interfaces.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuild>)` - Vector of guild models the user is a member of
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_guilds_for_user(&self, user_id: u64) -> Result<Vec<DiscordGuild>, DbErr> {
        let entities = entity::prelude::DiscordGuild::find()
            .inner_join(entity::prelude::DiscordGuildMember)
            .filter(entity::discord_guild_member::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?;

        entities
            .into_iter()
            .map(DiscordGuild::from_entity)
            .collect()
    }

    /// Checks if a guild needs a full sync based on the last sync timestamp.
    ///
    /// Returns true if the guild hasn't been synced in the last 30 minutes,
    /// preventing excessive syncs on frequent bot restarts or guild availability events.
    /// Used to determine whether to perform a full sync (roles, channels, members)
    /// when the bot starts or rejoins a guild.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(true)` - Guild needs sync (never synced or > 30 minutes since last sync)
    /// - `Ok(false)` - Guild was synced recently, skip sync
    /// - `Err(DbErr)` - Database error during query
    pub async fn needs_sync(&self, guild_id: u64) -> Result<bool, DbErr> {
        let guild = self.find_by_guild_id(guild_id).await?;

        match guild {
            Some(guild) => {
                let now = Utc::now();
                let sync_threshold = Duration::minutes(30);
                let needs_sync = now.signed_duration_since(guild.last_sync_at) > sync_threshold;
                Ok(needs_sync)
            }
            None => Ok(true), // Guild not in DB, needs sync
        }
    }

    /// Updates the last sync timestamp for a guild to the current time.
    ///
    /// Called after successfully completing a full guild sync (roles, channels, members).
    /// This prevents redundant syncs by marking when the guild data was last refreshed.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(())` - Timestamp updated successfully
    /// - `Err(DbErr)` - Database error during update
    pub async fn update_last_sync(&self, guild_id: u64) -> Result<(), DbErr> {
        entity::prelude::DiscordGuild::update_many()
            .filter(entity::discord_guild::Column::GuildId.eq(guild_id.to_string()))
            .col_expr(
                entity::discord_guild::Column::LastSyncAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .exec(self.db)
            .await?;

        Ok(())
    }
}
