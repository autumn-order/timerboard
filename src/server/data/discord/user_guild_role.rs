use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub struct UserDiscordGuildRoleRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildRoleRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a relationship between a user and a guild role
    ///
    /// Establishes that the specified user has the specified role in a guild.
    /// Does not check if the relationship already exists.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `role_id`: Discord role ID (u64)
    ///
    /// # Returns
    /// - `Ok(Model)`: The created user-guild-role relationship
    /// - `Err(DbErr)`: Database error (e.g., foreign key constraint violation)
    pub async fn create(
        &self,
        user_id: u64,
        role_id: u64,
    ) -> Result<entity::user_discord_guild_role::Model, DbErr> {
        entity::prelude::UserDiscordGuildRole::insert(
            entity::user_discord_guild_role::ActiveModel {
                user_id: ActiveValue::Set(user_id.to_string()),
                role_id: ActiveValue::Set(role_id.to_string()),
            },
        )
        .exec_with_returning(self.db)
        .await
    }

    /// Creates multiple user-guild-role relationships for a single user
    ///
    /// Establishes relationships between the user and multiple guild roles. Checks for existing
    /// relationships before creating new ones to avoid duplicates.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `role_ids`: Slice of Discord role IDs
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of newly created relationships (excludes existing ones)
    /// - `Err(DbErr)`: Database error during creation
    pub async fn create_many(
        &self,
        user_id: u64,
        role_ids: &[u64],
    ) -> Result<Vec<entity::user_discord_guild_role::Model>, DbErr> {
        let mut results = Vec::new();

        let user_id_str = user_id.to_string();
        for role_id in role_ids {
            // Check if relationship already exists
            let role_id_str = role_id.to_string();
            let exists = entity::prelude::UserDiscordGuildRole::find()
                .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id_str.as_str()))
                .filter(entity::user_discord_guild_role::Column::RoleId.eq(role_id_str.as_str()))
                .one(self.db)
                .await?;

            if exists.is_none() {
                let model = self.create(user_id, *role_id).await?;
                results.push(model);
            }
        }

        Ok(results)
    }

    /// Deletes all guild role relationships for a specific user
    ///
    /// Removes all role memberships for the user across all guilds. Used when cleaning up
    /// user data or when re-syncing all of a user's roles from scratch.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    ///
    /// # Returns
    /// - `Ok(())`: All relationships successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete_by_user(&self, user_id: u64) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuildRole::delete_many()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Deletes a specific user-guild-role relationship
    ///
    /// Removes the relationship indicating that the user has the specified role.
    /// Called when a user loses a role in Discord.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `role_id`: Discord role ID (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Relationship successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete(&self, user_id: u64, role_id: u64) -> Result<(), DbErr> {
        let user_id_str = user_id.to_string();
        let role_id_str = role_id.to_string();
        entity::prelude::UserDiscordGuildRole::delete_many()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id_str.as_str()))
            .filter(entity::user_discord_guild_role::Column::RoleId.eq(role_id_str.as_str()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Syncs user's guild role memberships by removing old ones and adding new ones
    ///
    /// Replaces all role memberships for a user with the provided list. First deletes
    /// all existing role relationships, then creates new ones for the provided Discord role IDs.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `role_ids`: Slice of Discord role IDs the user should have
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(DbErr)`: Database error during deletion or creation
    pub async fn sync_user_roles(&self, user_id: u64, role_ids: &[u64]) -> Result<(), DbErr> {
        // Delete all existing role relationships for this user
        self.delete_by_user(user_id).await?;

        // Create new relationships
        self.create_many(user_id, role_ids).await?;

        Ok(())
    }
}
