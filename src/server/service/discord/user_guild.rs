use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::GuildId;

use crate::server::{
    data::{
        discord::{DiscordGuildRepository, UserDiscordGuildRepository},
        user::UserRepository,
    },
    error::AppError,
};

pub struct UserDiscordGuildService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Syncs a user's guild memberships with guilds the bot is present in
    ///
    /// Compares the user's Discord guild memberships with guilds in the database (where the bot is present).
    /// Only creates relationships for guilds where both the user and bot are members. Replaces all existing
    /// guild memberships for the user.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    /// - `discord_user_id`: Discord's unique identifier for the user (for logging)
    /// - `user_guild_ids`: Slice of Discord guild IDs the user is a member of
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(AppError)`: Database error during guild query or sync
    pub async fn sync_user_guilds(
        &self,
        user_id: i32,
        discord_user_id: u64,
        user_guild_ids: &[GuildId],
    ) -> Result<(), AppError> {
        let guild_repo = DiscordGuildRepository::new(self.db);
        let user_guild_repo = UserDiscordGuildRepository::new(self.db);

        // Get all guilds the bot is in
        let bot_guilds = guild_repo.get_all().await?;

        // Find matching guilds (where both user and bot are members)
        let matching_guilds: Vec<_> = bot_guilds
            .iter()
            .filter(|bot_guild| {
                // Parse guild_id from String to u64 for comparison
                if let Ok(guild_id_u64) = bot_guild.guild_id.parse::<u64>() {
                    user_guild_ids
                        .iter()
                        .any(|user_guild_id| user_guild_id.get() == guild_id_u64)
                } else {
                    false
                }
            })
            .collect();

        let matching_discord_guild_ids: Vec<u64> = matching_guilds
            .iter()
            .filter_map(|g| g.guild_id.parse::<u64>().ok())
            .collect();

        // Sync the user's guild memberships
        user_guild_repo
            .sync_user_guilds(user_id, &matching_discord_guild_ids)
            .await?;

        tracing::debug!(
            "Synced {} guild memberships for user {} (guilds: {:?})",
            matching_discord_guild_ids.len(),
            discord_user_id,
            matching_discord_guild_ids
        );

        Ok(())
    }

    /// Syncs members of a guild with logged-in users
    ///
    /// Updates the database to reflect which logged-in users are currently members of the guild.
    /// Removes relationships for users no longer in the guild and adds relationships for new members.
    /// Only processes users who have logged into the application. Used during bot startup to catch
    /// missed member join/leave events while the bot was offline. Updates the last_guild_sync_at
    /// timestamp for all synced users.
    ///
    /// # Arguments
    /// - `guild_id`: Discord's unique identifier for the guild (u64)
    /// - `member_discord_ids`: Slice of Discord user IDs currently in the guild
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully and timestamps updated
    /// - `Err(AppError)`: Database error during user query, guild query, or relationship sync
    pub async fn sync_guild_members(
        &self,
        guild_id: u64,
        member_discord_ids: &[u64],
    ) -> Result<(), AppError> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let guild_repo = DiscordGuildRepository::new(self.db);
        let user_guild_repo = UserDiscordGuildRepository::new(self.db);

        tracing::debug!("Syncing members for guild {}", guild_id);

        // Get the guild from database
        let Some(guild) = guild_repo.find_by_guild_id(guild_id).await? else {
            tracing::warn!(
                "Guild {} not found in database during member sync",
                guild_id
            );
            return Ok(());
        };

        // Get all logged-in users who are members of this Discord guild
        let logged_in_members: Vec<entity::user::Model> = entity::prelude::User::find()
            .filter(
                entity::user::Column::DiscordId.is_in(
                    member_discord_ids
                        .iter()
                        .map(|id| *id as i64)
                        .collect::<Vec<_>>(),
                ),
            )
            .all(self.db)
            .await?;

        if logged_in_members.is_empty() {
            tracing::debug!(
                "Found no logged in users for guild {}, nothing to sync",
                guild_id
            );

            // No logged-in users in this guild, nothing to sync
            return Ok(());
        }

        // Get existing relationships for this guild
        let guild_id_u64 = guild
            .guild_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Failed to parse guild_id: {}", e)))?;
        let existing_relationships = user_guild_repo.get_users_by_guild(guild_id_u64).await?;
        let existing_user_ids: std::collections::HashSet<i32> =
            existing_relationships.iter().map(|r| r.user_id).collect();

        let logged_in_user_ids: std::collections::HashSet<i32> =
            logged_in_members.iter().map(|u| u.id).collect();

        // Collect synced user IDs before moving logged_in_members
        let synced_user_ids: Vec<i32> = logged_in_members.iter().map(|u| u.id).collect();

        // Remove relationships for users who are no longer in the guild
        for relationship in existing_relationships {
            if !logged_in_user_ids.contains(&relationship.user_id) {
                user_guild_repo
                    .delete(relationship.user_id, guild_id_u64)
                    .await?;
            }
        }

        // Add relationships for users who are in the guild but not in our database
        for user in logged_in_members {
            if !existing_user_ids.contains(&user.id) {
                user_guild_repo.create(user.id, guild_id_u64).await?;
            }
        }

        tracing::info!("Synced members for guild {} ({})", guild.name, guild_id);

        // Update last_guild_sync_at timestamps for all synced users
        if !synced_user_ids.is_empty() {
            let user_repo = UserRepository::new(self.db);
            user_repo
                .update_guild_sync_timestamps(&synced_user_ids)
                .await?;
        }

        Ok(())
    }
}
