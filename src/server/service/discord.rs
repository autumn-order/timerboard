use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{Role, RoleId};
use std::collections::HashMap;

use crate::{
    model::discord::DiscordGuildDto,
    server::{
        data::discord::{DiscordGuildRepository, DiscordGuildRoleRepository},
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
