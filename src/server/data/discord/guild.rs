use chrono::{Duration, Utc};
use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::Guild;

pub struct DiscordGuildRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, guild: Guild) -> Result<entity::discord_guild::Model, DbErr> {
        entity::prelude::DiscordGuild::insert(entity::discord_guild::ActiveModel {
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
        .await
    }

    pub async fn get_all(&self) -> Result<Vec<entity::discord_guild::Model>, DbErr> {
        entity::prelude::DiscordGuild::find().all(self.db).await
    }

    /// Finds a guild by its Discord guild ID
    ///
    /// Searches for a guild in the database using the Discord-assigned guild ID.
    /// Used to check if the bot is present in a specific guild.
    ///
    /// # Arguments
    /// - `guild_id` - Discord's unique identifier for the guild (u64)
    ///
    /// # Returns
    /// - `Ok(Some(Model))` - Guild found in database
    /// - `Ok(None)` - Guild not found (bot not in this guild)
    /// - `Err(DbErr)` - Database error during query
    pub async fn find_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Option<entity::discord_guild::Model>, DbErr> {
        entity::prelude::DiscordGuild::find()
            .filter(entity::discord_guild::Column::GuildId.eq(guild_id.to_string()))
            .one(self.db)
            .await
    }

    /// Gets all guilds for a specific user
    ///
    /// Retrieves all guilds that the specified user is a member of.
    /// Used to determine which timerboards are available to a user.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID (u64)
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)` - Vector of guild models the user is a member of
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_guilds_for_user(
        &self,
        user_id: u64,
    ) -> Result<Vec<entity::discord_guild::Model>, DbErr> {
        entity::prelude::DiscordGuild::find()
            .inner_join(entity::prelude::DiscordGuildMember)
            .filter(entity::discord_guild_member::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await
    }

    /// Checks if a guild needs a full sync based on the last sync timestamp
    ///
    /// Returns true if the guild hasn't been synced in the last 30 minutes,
    /// preventing excessive syncs on frequent bot restarts.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID (u64)
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

    /// Updates the last sync timestamp for a guild
    ///
    /// Called after successfully completing a full guild sync (roles, channels, members).
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID (u64)
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
