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
    /// - `guild_id`: Discord's unique identifier for the guild (u64)
    ///
    /// # Returns
    /// - `Ok(Some(Model))`: Guild found in database
    /// - `Ok(None)`: Guild not found (bot not in this guild)
    /// - `Err(DbErr)`: Database error during query
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
    /// - `user_id`: Discord user ID (u64)
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of guild models the user is a member of
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_guilds_for_user(
        &self,
        user_id: u64,
    ) -> Result<Vec<entity::discord_guild::Model>, DbErr> {
        entity::prelude::DiscordGuild::find()
            .inner_join(entity::prelude::UserDiscordGuild)
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await
    }
}
