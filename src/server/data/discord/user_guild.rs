use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub struct UserDiscordGuildRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a relationship between a user and a guild
    ///
    /// Establishes that the specified user is a member of the specified guild.
    /// Does not check if the relationship already exists.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_id`: Discord guild ID (u64)
    ///
    /// # Returns
    /// - `Ok(Model)`: The created user-guild relationship
    /// - `Err(DbErr)`: Database error (e.g., foreign key constraint violation)
    pub async fn create(
        &self,
        user_id: u64,
        guild_id: u64,
        nickname: Option<String>,
    ) -> Result<entity::user_discord_guild::Model, DbErr> {
        entity::prelude::UserDiscordGuild::insert(entity::user_discord_guild::ActiveModel {
            user_id: ActiveValue::Set(user_id.to_string()),
            guild_id: ActiveValue::Set(guild_id.to_string()),
            nickname: ActiveValue::Set(nickname),
        })
        .exec_with_returning(self.db)
        .await
    }

    /// Creates multiple user-guild relationships for a single user
    ///
    /// Establishes relationships between the user and multiple guilds. Checks for existing
    /// relationships before creating new ones to avoid duplicates.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_ids`: Slice of Discord guild IDs
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of newly created relationships (excludes existing ones)
    /// - `Err(DbErr)`: Database error during creation
    pub async fn create_many(
        &self,
        user_id: u64,
        guild_ids: &[(u64, Option<String>)],
    ) -> Result<Vec<entity::user_discord_guild::Model>, DbErr> {
        let mut results = Vec::new();

        let user_id_str = user_id.to_string();
        for (guild_id, nickname) in guild_ids {
            // Check if relationship already exists
            let guild_id_str = guild_id.to_string();
            let exists = entity::prelude::UserDiscordGuild::find()
                .filter(entity::user_discord_guild::Column::UserId.eq(user_id_str.as_str()))
                .filter(entity::user_discord_guild::Column::GuildId.eq(guild_id_str.as_str()))
                .one(self.db)
                .await?;

            if exists.is_none() {
                let model = self.create(user_id, *guild_id, nickname.clone()).await?;
                results.push(model);
            }
        }

        Ok(results)
    }

    /// Deletes all guild relationships for a specific user
    ///
    /// Removes all guild memberships for the user. Used when cleaning up user data
    /// or when re-syncing all of a user's guilds from scratch.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    ///
    /// # Returns
    /// - `Ok(())`: All relationships successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete_by_user(&self, user_id: u64) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuild::delete_many()
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id.to_string()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Deletes a specific user-guild relationship
    ///
    /// Removes the relationship indicating that the user is a member of the guild.
    /// Called when a user leaves a guild or is removed.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_id`: Discord guild ID (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Relationship successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete(&self, user_id: u64, guild_id: u64) -> Result<(), DbErr> {
        let user_id_str = user_id.to_string();
        let guild_id_str = guild_id.to_string();
        entity::prelude::UserDiscordGuild::delete_many()
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id_str.as_str()))
            .filter(entity::user_discord_guild::Column::GuildId.eq(guild_id_str.as_str()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Gets all users for a specific guild
    ///
    /// Retrieves all user-guild relationships for a given guild. Used to determine
    /// which logged-in users are members of a particular guild.
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of user-guild relationships for the guild
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_users_by_guild(
        &self,
        guild_id: u64,
    ) -> Result<Vec<entity::user_discord_guild::Model>, DbErr> {
        let guild_id_str = guild_id.to_string();
        entity::prelude::UserDiscordGuild::find()
            .filter(entity::user_discord_guild::Column::GuildId.eq(guild_id_str.as_str()))
            .all(self.db)
            .await
    }

    /// Syncs user's guild memberships by removing old ones and adding new ones
    ///
    /// Replaces all guild memberships for a user with the provided list. First deletes
    /// all existing relationships, then creates new ones for the provided guild IDs.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_ids`: Slice of Discord guild IDs the user should be a member of
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(DbErr)`: Database error during deletion or creation
    pub async fn sync_user_guilds(
        &self,
        user_id: u64,
        guild_ids: &[(u64, Option<String>)],
    ) -> Result<(), DbErr> {
        // Delete all existing relationships for this user
        self.delete_by_user(user_id).await?;

        // Create new relationships
        self.create_many(user_id, guild_ids).await?;

        Ok(())
    }

    /// Gets all users with details and nicknames for a specific guild
    ///
    /// Retrieves user information for all members of a guild along with their guild-specific nicknames.
    /// Used for showing guild member lists.
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<(User, Option<String>)>)`: Vector of tuples containing user model and optional nickname
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_guild_members_with_nicknames(
        &self,
        guild_id: u64,
    ) -> Result<Vec<(entity::user::Model, Option<String>)>, DbErr> {
        // Get all user IDs for this guild with nicknames
        let user_guild_relationships = self.get_users_by_guild(guild_id).await?;

        if user_guild_relationships.is_empty() {
            return Ok(Vec::new());
        }

        let user_ids: Vec<String> = user_guild_relationships
            .iter()
            .map(|ug| ug.user_id.clone())
            .collect();

        // Fetch all user models
        let users = entity::prelude::User::find()
            .filter(entity::user::Column::DiscordId.is_in(user_ids))
            .all(self.db)
            .await?;

        // Map users with their nicknames
        let mut results = Vec::new();
        for user in users {
            let nickname = user_guild_relationships
                .iter()
                .find(|ug| ug.user_id == user.discord_id)
                .and_then(|ug| ug.nickname.clone());
            results.push((user, nickname));
        }

        Ok(results)
    }
}
