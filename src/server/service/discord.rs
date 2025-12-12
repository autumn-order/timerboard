use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{GuildId, Role, RoleId};
use std::collections::HashMap;

use crate::{
    model::discord::DiscordGuildDto,
    server::{
        data::discord::{
            DiscordGuildRepository, DiscordGuildRoleRepository, UserDiscordGuildRepository,
        },
        error::AppError,
    },
};

pub struct DiscordGuildService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_all(&self) -> Result<Vec<DiscordGuildDto>, AppError> {
        let guild_repo = DiscordGuildRepository::new(self.db);

        let guilds = guild_repo
            .get_all()
            .await?
            .into_iter()
            .map(|g| DiscordGuildDto {
                id: g.id,
                guild_id: g.guild_id,
                name: g.name,
                icon_hash: g.icon_hash,
            })
            .collect();

        Ok(guilds)
    }
}

pub struct DiscordGuildRoleService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRoleService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Updates roles for a guild by deleting roles that no longer exist and upserting current roles
    pub async fn update_roles(
        &self,
        guild_id: u64,
        guild_roles: &HashMap<RoleId, Role>,
    ) -> Result<(), AppError> {
        let role_repo = DiscordGuildRoleRepository::new(self.db);

        // Get existing roles from database
        let existing_roles = role_repo.get_by_guild_id(guild_id).await?;

        // Find roles that no longer exist in Discord and delete them
        for existing_role in existing_roles {
            let role_id = existing_role.role_id as u64;
            if !guild_roles.contains_key(&RoleId::new(role_id)) {
                role_repo.delete(role_id).await?;
                tracing::info!("Deleted role {} from guild {}", role_id, guild_id);
            }
        }

        // Upsert all current roles
        role_repo.upsert_many(guild_id, guild_roles).await?;

        tracing::info!("Updated {} roles for guild {}", guild_roles.len(), guild_id);

        Ok(())
    }
}

pub struct UserDiscordGuildService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserDiscordGuildService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Syncs a user's guild memberships with guilds the bot is present in
    /// Only creates relationships for guilds where both the user and bot are members
    pub async fn sync_user_guilds(
        &self,
        user_id: i32,
        user_guild_ids: &[GuildId],
    ) -> Result<(), AppError> {
        let guild_repo = DiscordGuildRepository::new(self.db);
        let user_guild_repo = UserDiscordGuildRepository::new(self.db);

        // Get all guilds the bot is in
        let bot_guilds = guild_repo.get_all().await?;

        // Find matching guilds (where both user and bot are members)
        let matching_guild_ids: Vec<i32> = bot_guilds
            .iter()
            .filter(|bot_guild| {
                user_guild_ids
                    .iter()
                    .any(|user_guild_id| user_guild_id.get() == bot_guild.guild_id as u64)
            })
            .map(|guild| guild.id)
            .collect();

        // Sync the user's guild memberships
        user_guild_repo
            .sync_user_guilds(user_id, &matching_guild_ids)
            .await?;

        tracing::info!(
            "Synced {} guild memberships for user {}",
            matching_guild_ids.len(),
            user_id
        );

        Ok(())
    }

    /// Syncs members of a guild with logged-in users
    /// Used during guild_create to catch up on missed member join/leave events
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
        let existing_relationships = user_guild_repo.get_users_by_guild(guild.id).await?;
        let existing_user_ids: std::collections::HashSet<i32> =
            existing_relationships.iter().map(|r| r.user_id).collect();

        let logged_in_user_ids: std::collections::HashSet<i32> =
            logged_in_members.iter().map(|u| u.id).collect();

        // Remove relationships for users who are no longer in the guild
        for relationship in existing_relationships {
            if !logged_in_user_ids.contains(&relationship.user_id) {
                user_guild_repo
                    .delete(relationship.user_id, guild.id)
                    .await?;
            }
        }

        // Add relationships for users who are in the guild but not in our database
        for user in logged_in_members {
            if !existing_user_ids.contains(&user.id) {
                user_guild_repo.create(user.id, guild.id).await?;
            }
        }

        tracing::info!("Synced members for guild {} ({})", guild.name, guild_id);

        Ok(())
    }
}
