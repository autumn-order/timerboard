use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

pub struct DiscordGuildMemberRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildMemberRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates or updates a guild member record
    ///
    /// Stores information about a Discord user who is a member of a guild.
    /// This tracks ALL members, not just those with application accounts.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_id`: Discord guild ID (u64)
    /// - `username`: Discord username
    /// - `nickname`: Optional guild-specific nickname
    ///
    /// # Returns
    /// - `Ok(Model)`: The created/updated guild member record
    /// - `Err(DbErr)`: Database error
    pub async fn upsert(
        &self,
        user_id: u64,
        guild_id: u64,
        username: String,
        nickname: Option<String>,
    ) -> Result<entity::discord_guild_member::Model, DbErr> {
        let user_id_str = user_id.to_string();
        let guild_id_str = guild_id.to_string();

        // Check if member already exists
        let existing = entity::prelude::DiscordGuildMember::find()
            .filter(entity::discord_guild_member::Column::UserId.eq(&user_id_str))
            .filter(entity::discord_guild_member::Column::GuildId.eq(&guild_id_str))
            .one(self.db)
            .await?;

        if let Some(existing) = existing {
            // Update existing record
            let mut active: entity::discord_guild_member::ActiveModel = existing.into();
            active.username = ActiveValue::Set(username);
            active.nickname = ActiveValue::Set(nickname);
            active.update(self.db).await
        } else {
            // Create new record
            entity::prelude::DiscordGuildMember::insert(entity::discord_guild_member::ActiveModel {
                user_id: ActiveValue::Set(user_id_str),
                guild_id: ActiveValue::Set(guild_id_str),
                username: ActiveValue::Set(username),
                nickname: ActiveValue::Set(nickname),
            })
            .exec_with_returning(self.db)
            .await
        }
    }

    /// Deletes a guild member record
    ///
    /// Removes a member from the guild member tracking. Called when a user leaves a guild.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID (u64)
    /// - `guild_id`: Discord guild ID (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Member record successfully deleted
    /// - `Err(DbErr)`: Database error during deletion
    pub async fn delete(&self, user_id: u64, guild_id: u64) -> Result<(), DbErr> {
        let user_id_str = user_id.to_string();
        let guild_id_str = guild_id.to_string();
        entity::prelude::DiscordGuildMember::delete_many()
            .filter(entity::discord_guild_member::Column::UserId.eq(user_id_str.as_str()))
            .filter(entity::discord_guild_member::Column::GuildId.eq(guild_id_str.as_str()))
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Gets all members for a specific guild
    ///
    /// Retrieves all guild members (Discord users) for a given guild.
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of guild member records
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_members_by_guild(
        &self,
        guild_id: u64,
    ) -> Result<Vec<entity::discord_guild_member::Model>, DbErr> {
        let guild_id_str = guild_id.to_string();
        entity::prelude::DiscordGuildMember::find()
            .filter(entity::discord_guild_member::Column::GuildId.eq(guild_id_str.as_str()))
            .all(self.db)
            .await
    }

    /// Gets a specific member record
    ///
    /// Retrieves a single member's record for a guild.
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID
    /// - `guild_id`: Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Option<Model>)`: The member record if found
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_member(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Option<entity::discord_guild_member::Model>, DbErr> {
        let user_id_str = user_id.to_string();
        let guild_id_str = guild_id.to_string();
        entity::prelude::DiscordGuildMember::find()
            .filter(entity::discord_guild_member::Column::UserId.eq(user_id_str.as_str()))
            .filter(entity::discord_guild_member::Column::GuildId.eq(guild_id_str.as_str()))
            .one(self.db)
            .await
    }

    /// Syncs guild members by removing members not in the provided list and adding/updating new ones
    ///
    /// This is a full sync operation that ensures the database matches the provided member list.
    /// Removes members who are no longer in the guild and adds/updates current members.
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID
    /// - `members`: Slice of tuples containing (user_id, username, nickname)
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(DbErr)`: Database error during sync
    pub async fn sync_guild_members(
        &self,
        guild_id: u64,
        members: &[(u64, String, Option<String>)],
    ) -> Result<(), DbErr> {
        // Get current members in database
        let current_members = self.get_members_by_guild(guild_id).await?;
        let _current_user_ids: Vec<String> =
            current_members.iter().map(|m| m.user_id.clone()).collect();

        // Determine which members to keep
        let new_user_ids: Vec<String> = members.iter().map(|(id, _, _)| id.to_string()).collect();

        // Delete members who are no longer in the guild
        for current_member in current_members {
            if !new_user_ids.contains(&current_member.user_id) {
                let user_id = current_member.user_id.parse::<u64>().unwrap_or(0);
                self.delete(user_id, guild_id).await?;
            }
        }

        // Upsert all current members
        for (user_id, username, nickname) in members {
            self.upsert(*user_id, guild_id, username.clone(), nickname.clone())
                .await?;
        }

        Ok(())
    }
}
