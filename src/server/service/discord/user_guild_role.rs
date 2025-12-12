use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::Member;

use crate::server::{
    data::discord::{DiscordGuildRoleRepository, UserDiscordGuildRoleRepository},
    error::AppError,
};

pub struct UserDiscordGuildRoleService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildRoleService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Syncs a user's role memberships for a specific guild
    ///
    /// Updates the database to reflect which roles the user currently has in the guild.
    /// Only creates relationships for roles that exist in the database (tracked by the bot).
    /// Replaces all existing role memberships for the user with the current set.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    /// - `member`: Discord Member object containing the user's current roles
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully
    /// - `Err(AppError)`: Database error during role query or sync
    pub async fn sync_user_roles(&self, user_id: i32, member: &Member) -> Result<(), AppError> {
        let role_repo = DiscordGuildRoleRepository::new(self.db);
        let user_role_repo = UserDiscordGuildRoleRepository::new(self.db);

        // Get all roles from database for this guild
        let guild_id = member.guild_id.get();
        let discord_user_id = member.user.id.get();
        let db_roles = role_repo.get_by_guild_id(guild_id).await?;

        // Find matching role IDs (roles the user has that are in our database)
        let user_role_ids: Vec<u64> = member.roles.iter().map(|r| r.get()).collect();

        let matching_guild_role_ids: Vec<i32> = db_roles
            .iter()
            .filter(|db_role| user_role_ids.contains(&(db_role.role_id as u64)))
            .map(|role| role.id)
            .collect();

        // Sync the user's role memberships
        user_role_repo
            .sync_user_guild_roles(user_id, &matching_guild_role_ids)
            .await?;

        tracing::debug!(
            "Synced {} role memberships for user {} in guild {}",
            matching_guild_role_ids.len(),
            discord_user_id,
            guild_id
        );

        Ok(())
    }

    /// Syncs role memberships for all logged-in users in a guild
    ///
    /// Updates role memberships for all users who have logged into the application and are
    /// members of the specified guild. Only processes logged-in users and only syncs roles
    /// that exist in the database. Used during bot startup to catch missed role changes
    /// while the bot was offline.
    ///
    /// # Arguments
    /// - `guild_id`: Discord's unique identifier for the guild (u64)
    /// - `members`: Slice of Discord Member objects for users in the guild
    ///
    /// # Returns
    /// - `Ok(())`: Sync completed successfully for all applicable users
    /// - `Err(AppError)`: Database error during user query or role sync
    pub async fn sync_guild_member_roles(
        &self,
        guild_id: u64,
        members: &[Member],
    ) -> Result<(), AppError> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        tracing::debug!(
            "Syncing role memberships for guild {} ({} members)",
            guild_id,
            members.len()
        );

        // Get all logged-in users
        let member_discord_ids: Vec<i64> = members.iter().map(|m| m.user.id.get() as i64).collect();

        let logged_in_users: Vec<entity::user::Model> = entity::prelude::User::find()
            .filter(entity::user::Column::DiscordId.is_in(member_discord_ids))
            .all(self.db)
            .await?;

        if logged_in_users.is_empty() {
            tracing::debug!(
                "No logged-in users found in guild {}, skipping role sync",
                guild_id
            );
            return Ok(());
        }

        // Sync roles for each logged-in user
        let mut synced_count = 0;
        for user in logged_in_users {
            // Find the corresponding member
            if let Some(member) = members
                .iter()
                .find(|m| m.user.id.get() == user.discord_id as u64)
            {
                if let Err(e) = self.sync_user_roles(user.id, member).await {
                    tracing::error!(
                        "Failed to sync roles for user {} in guild {}: {:?}",
                        user.id,
                        guild_id,
                        e
                    );
                } else {
                    synced_count += 1;
                }
            }
        }

        tracing::debug!(
            "Synced role memberships for {} users in guild {}",
            synced_count,
            guild_id
        );

        Ok(())
    }

    /// Adds a single role to a user
    ///
    /// Creates a relationship indicating the user has the specified role. Looks up the role
    /// in the database by its Discord role ID and creates the relationship if found.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    /// - `role_id`: Discord's unique identifier for the role (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Role relationship created successfully
    /// - `Err(AppError)`: Database error during role lookup or creation
    #[allow(dead_code)]
    pub async fn add_user_role(&self, user_id: i32, role_id: u64) -> Result<(), AppError> {
        let user_role_repo = UserDiscordGuildRoleRepository::new(self.db);

        // Find by role_id across all guilds
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let guild_role = entity::prelude::DiscordGuildRole::find()
            .filter(entity::discord_guild_role::Column::RoleId.eq(role_id as i64))
            .one(self.db)
            .await?;

        if let Some(guild_role) = guild_role {
            user_role_repo.create(user_id, guild_role.id).await?;
            tracing::info!("Added role {} to user {}", role_id, user_id);
        } else {
            tracing::warn!(
                "Role {} not found in database when trying to add to user {}",
                role_id,
                user_id
            );
        }

        Ok(())
    }

    /// Removes a single role from a user
    ///
    /// Deletes the relationship indicating the user has the specified role. Looks up the role
    /// in the database by its Discord role ID and removes the relationship if found.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    /// - `role_id`: Discord's unique identifier for the role (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Role relationship removed successfully
    /// - `Err(AppError)`: Database error during role lookup or deletion
    #[allow(dead_code)]
    pub async fn remove_user_role(&self, user_id: i32, role_id: u64) -> Result<(), AppError> {
        let user_role_repo = UserDiscordGuildRoleRepository::new(self.db);

        // Find the guild role in database
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let guild_role = entity::prelude::DiscordGuildRole::find()
            .filter(entity::discord_guild_role::Column::RoleId.eq(role_id as i64))
            .one(self.db)
            .await?;

        if let Some(guild_role) = guild_role {
            user_role_repo.delete(user_id, guild_role.id).await?;
            tracing::info!("Removed role {} from user {}", role_id, user_id);
        } else {
            tracing::debug!(
                "Role {} not found in database when trying to remove from user {}",
                role_id,
                user_id
            );
        }

        Ok(())
    }
}
