//! Discord guild member repository for database operations.
//!
//! This module provides the `DiscordGuildMemberRepository` for managing Discord
//! guild member records in the database. It tracks all guild members (not just
//! those with application accounts), storing their username and guild-specific
//! nickname. This data is synced from Discord and used for display purposes.
//!
//! All methods return domain models at the repository boundary, converting SeaORM
//! entity models internally to prevent database-specific structures from leaking
//! into service and controller layers.

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

use crate::server::model::discord::DiscordGuildMember;

/// Repository for Discord guild member database operations.
///
/// Provides methods for upserting, deleting, and querying guild members.
/// Used to keep local member data synchronized with Discord's state for
/// display and identification purposes.
pub struct DiscordGuildMemberRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildMemberRepository<'a> {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    /// - `db` - Database connection for executing queries
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates or updates a guild member record.
    ///
    /// Stores information about a Discord user who is a member of a guild.
    /// This tracks ALL members, not just those with application accounts.
    /// If the member already exists, updates their username and nickname.
    /// Otherwise, creates a new member record.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    /// - `username` - Discord username
    /// - `nickname` - Optional guild-specific nickname
    ///
    /// # Returns
    /// - `Ok(DiscordGuildMemberParam)` - The created/updated guild member record
    /// - `Err(DbErr)` - Database error during query or upsert
    pub async fn upsert(
        &self,
        user_id: u64,
        guild_id: u64,
        username: String,
        nickname: Option<String>,
    ) -> Result<DiscordGuildMember, DbErr> {
        let user_id_str = user_id.to_string();
        let guild_id_str = guild_id.to_string();

        // Check if member already exists
        let existing = entity::prelude::DiscordGuildMember::find()
            .filter(entity::discord_guild_member::Column::UserId.eq(&user_id_str))
            .filter(entity::discord_guild_member::Column::GuildId.eq(&guild_id_str))
            .one(self.db)
            .await?;

        let entity = if let Some(existing) = existing {
            // Update existing record
            let mut active: entity::discord_guild_member::ActiveModel = existing.into();
            active.username = ActiveValue::Set(username);
            active.nickname = ActiveValue::Set(nickname);
            active.update(self.db).await?
        } else {
            // Create new record
            entity::prelude::DiscordGuildMember::insert(entity::discord_guild_member::ActiveModel {
                user_id: ActiveValue::Set(user_id_str),
                guild_id: ActiveValue::Set(guild_id_str),
                username: ActiveValue::Set(username),
                nickname: ActiveValue::Set(nickname),
            })
            .exec_with_returning(self.db)
            .await?
        };

        DiscordGuildMember::from_entity(entity)
    }

    /// Deletes a guild member record.
    ///
    /// Removes a member from the guild member tracking. Called when a user leaves
    /// a guild or is kicked/banned. No-op if the member record doesn't exist.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(())` - Member record successfully deleted (or didn't exist)
    /// - `Err(DbErr)` - Database error during deletion
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

    /// Gets all members for a specific guild.
    ///
    /// Retrieves all guild members (Discord users) for a given guild. Used for
    /// syncing operations and displaying member lists in administrative interfaces.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuildMemberParam>)` - Vector of guild member records
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_members_by_guild(
        &self,
        guild_id: u64,
    ) -> Result<Vec<DiscordGuildMember>, DbErr> {
        let guild_id_str = guild_id.to_string();
        let entities = entity::prelude::DiscordGuildMember::find()
            .filter(entity::discord_guild_member::Column::GuildId.eq(guild_id_str.as_str()))
            .all(self.db)
            .await?;

        entities
            .into_iter()
            .map(DiscordGuildMember::from_entity)
            .collect()
    }

    /// Gets a specific member record.
    ///
    /// Retrieves a single member's record for a guild. Used for looking up
    /// member details when displaying user information or checking membership status.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Some(DiscordGuildMemberParam))` - The member record if found
    /// - `Ok(None)` - Member not found
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_member(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Option<DiscordGuildMember>, DbErr> {
        let user_id_str = user_id.to_string();
        let guild_id_str = guild_id.to_string();
        let entity = entity::prelude::DiscordGuildMember::find()
            .filter(entity::discord_guild_member::Column::UserId.eq(user_id_str.as_str()))
            .filter(entity::discord_guild_member::Column::GuildId.eq(guild_id_str.as_str()))
            .one(self.db)
            .await?;

        entity.map(DiscordGuildMember::from_entity).transpose()
    }

    /// Syncs guild members by replacing all members with the provided list.
    ///
    /// This is a full sync operation that ensures the database matches the provided
    /// member list. Removes members who are no longer in the guild and adds/updates
    /// current members. Used when fetching all members from Discord to ensure local
    /// state matches Discord's state.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID
    /// - `members` - Slice of tuples containing (user_id, username, nickname)
    ///
    /// # Returns
    /// - `Ok(())` - Sync completed successfully
    /// - `Err(DbErr)` - Database error during deletion or upsert operations
    pub async fn sync_guild_members(
        &self,
        guild_id: u64,
        members: &[(u64, String, Option<String>)],
    ) -> Result<(), DbErr> {
        // Get current members in database
        let current_members = self.get_members_by_guild(guild_id).await?;

        // Determine which members to keep
        let new_user_ids: Vec<u64> = members.iter().map(|(id, _, _)| *id).collect();

        // Delete members who are no longer in the guild
        for current_member in current_members {
            if !new_user_ids.contains(&current_member.user_id) {
                self.delete(current_member.user_id, guild_id).await?;
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
