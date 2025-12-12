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
    /// - `user_id`: Database ID of the user
    /// - `guild_id`: Database ID of the guild
    ///
    /// # Returns
    /// - `Ok(Model)`: The created user-guild relationship
    /// - `Err(DbErr)`: Database error (e.g., foreign key constraint violation)
    pub async fn create(
        &self,
        user_id: i32,
        guild_id: i32,
    ) -> Result<entity::user_discord_guild::Model, DbErr> {
        entity::prelude::UserDiscordGuild::insert(entity::user_discord_guild::ActiveModel {
            user_id: ActiveValue::Set(user_id),
            guild_id: ActiveValue::Set(guild_id),
            ..Default::default()
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
    /// - `user_id`: Database ID of the user
    /// - `guild_ids`: Slice of database IDs for the guilds
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of newly created relationships (excludes existing ones)
    /// - `Err(DbErr)`: Database error during creation
    pub async fn create_many(
        &self,
        user_id: i32,
        guild_ids: &[i32],
    ) -> Result<Vec<entity::user_discord_guild::Model>, DbErr> {
        let mut results = Vec::new();

        for guild_id in guild_ids {
            // Check if relationship already exists
            let exists = entity::prelude::UserDiscordGuild::find()
                .filter(entity::user_discord_guild::Column::UserId.eq(user_id))
                .filter(entity::user_discord_guild::Column::GuildId.eq(*guild_id))
                .one(self.db)
                .await?;

            if exists.is_none() {
                let model = self.create(user_id, *guild_id).await?;
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
    /// - `user_id`: Database ID of the user
    ///
    /// # Returns
    /// - `Ok(())`: All relationships successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete_by_user(&self, user_id: i32) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuild::delete_many()
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id))
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
    /// - `user_id`: Database ID of the user
    /// - `guild_id`: Database ID of the guild
    ///
    /// # Returns
    /// - `Ok(())`: Relationship successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete(&self, user_id: i32, guild_id: i32) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuild::delete_many()
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id))
            .filter(entity::user_discord_guild::Column::GuildId.eq(guild_id))
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
    /// - `guild_id`: Database ID of the guild
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of user-guild relationships for the guild
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_users_by_guild(
        &self,
        guild_id: i32,
    ) -> Result<Vec<entity::user_discord_guild::Model>, DbErr> {
        entity::prelude::UserDiscordGuild::find()
            .filter(entity::user_discord_guild::Column::GuildId.eq(guild_id))
            .all(self.db)
            .await
    }

    /// Syncs user's guild memberships by removing old ones and adding new ones
    ///
    /// Replaces all guild memberships for a user with the provided list. First deletes
    /// all existing relationships, then creates new ones for the provided guild IDs.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    /// - `guild_ids`: Slice of database IDs for guilds the user should be a member of
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(DbErr)`: Database error during deletion or creation
    pub async fn sync_user_guilds(&self, user_id: i32, guild_ids: &[i32]) -> Result<(), DbErr> {
        // Delete all existing relationships for this user
        self.delete_by_user(user_id).await?;

        // Create new relationships
        self.create_many(user_id, guild_ids).await?;

        Ok(())
    }
}
