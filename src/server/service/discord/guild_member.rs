use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;

use crate::server::{data::discord::DiscordGuildMemberRepository, error::AppError};

pub struct DiscordGuildMemberService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildMemberService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Syncs all members of a guild
    ///
    /// Updates the database to reflect the current membership of the guild.
    /// This stores ALL Discord users who are members, not just logged-in users.
    /// Removes members who have left and adds/updates current members.
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID (u64)
    /// - `members`: Slice of tuples containing (user_id, username, nickname)
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(AppError)`: Database error during sync
    pub async fn sync_guild_members(
        &self,
        guild_id: u64,
        members: &[(u64, String, Option<String>)],
    ) -> Result<(), AppError> {
        let member_repo = DiscordGuildMemberRepository::new(self.db);

        tracing::debug!("Syncing {} members for guild {}", members.len(), guild_id);

        member_repo.sync_guild_members(guild_id, members).await?;

        tracing::info!(
            "Successfully synced {} members for guild {}",
            members.len(),
            guild_id
        );

        Ok(())
    }

    /// Adds or updates a single guild member
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID
    /// - `guild_id`: Discord guild ID
    /// - `username`: Discord username
    /// - `nickname`: Optional guild-specific nickname
    ///
    /// # Returns
    /// - `Ok(())`: Member added/updated successfully
    /// - `Err(AppError)`: Database error
    pub async fn upsert_member(
        &self,
        user_id: u64,
        guild_id: u64,
        username: String,
        nickname: Option<String>,
    ) -> Result<(), AppError> {
        let member_repo = DiscordGuildMemberRepository::new(self.db);
        member_repo
            .upsert(user_id, guild_id, username, nickname)
            .await?;
        Ok(())
    }

    /// Removes a member from a guild
    ///
    /// # Arguments
    /// - `user_id`: Discord user ID
    /// - `guild_id`: Discord guild ID
    ///
    /// # Returns
    /// - `Ok(())`: Member removed successfully
    /// - `Err(AppError)`: Database error
    pub async fn remove_member(&self, user_id: u64, guild_id: u64) -> Result<(), AppError> {
        let member_repo = DiscordGuildMemberRepository::new(self.db);
        member_repo.delete(user_id, guild_id).await?;
        Ok(())
    }
}
