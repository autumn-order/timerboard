use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

pub struct UserDiscordGuildRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a relationship between a user and a guild
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

    /// Creates multiple user-guild relationships
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
    pub async fn delete_by_user(&self, user_id: i32) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuild::delete_many()
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Deletes a specific user-guild relationship
    pub async fn delete(&self, user_id: i32, guild_id: i32) -> Result<(), DbErr> {
        entity::prelude::UserDiscordGuild::delete_many()
            .filter(entity::user_discord_guild::Column::UserId.eq(user_id))
            .filter(entity::user_discord_guild::Column::GuildId.eq(guild_id))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Syncs user's guild memberships by removing old ones and adding new ones
    pub async fn sync_user_guilds(&self, user_id: i32, guild_ids: &[i32]) -> Result<(), DbErr> {
        // Delete all existing relationships for this user
        self.delete_by_user(user_id).await?;

        // Create new relationships
        self.create_many(user_id, guild_ids).await?;

        Ok(())
    }
}
