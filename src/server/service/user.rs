use sea_orm::DatabaseConnection;

use crate::{
    model::{
        discord::DiscordGuildDto,
        user::{PaginatedUsersDto, UserDto},
    },
    server::{
        data::{discord::guild::DiscordGuildRepository, user::UserRepository},
        error::AppError,
    },
};

pub struct UserService<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> UserService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_user(&self, user_id: u64) -> Result<Option<UserDto>, AppError> {
        let user_repo = UserRepository::new(self.db);

        let Some(user_model) = user_repo.find_by_discord_id(user_id).await? else {
            return Ok(None);
        };

        let user = UserDto {
            discord_id: user_model.discord_id.parse::<u64>().map_err(|e| {
                AppError::InternalError(format!("Failed to parse discord_id: {}", e))
            })?,
            name: user_model.name,
            admin: user_model.admin,
        };

        Ok(Some(user))
    }

    pub async fn get_all_users(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedUsersDto, AppError> {
        let user_repo = UserRepository::new(self.db);

        let (user_models, total_items) = user_repo.get_all_paginated(page, per_page).await?;

        let users = user_models
            .into_iter()
            .map(|user_model| {
                Ok(UserDto {
                    discord_id: user_model.discord_id.parse::<u64>().map_err(|e| {
                        AppError::InternalError(format!("Failed to parse discord_id: {}", e))
                    })?,
                    name: user_model.name,
                    admin: user_model.admin,
                })
            })
            .collect::<Result<Vec<UserDto>, AppError>>()?;

        let total_pages = (total_items as f64 / per_page as f64).ceil() as u64;

        Ok(PaginatedUsersDto {
            users,
            total: total_items,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn get_all_admins(&self) -> Result<Vec<UserDto>, AppError> {
        let user_repo = UserRepository::new(self.db);

        let user_models = user_repo.get_all_admins().await?;

        let users = user_models
            .into_iter()
            .map(|user_model| {
                Ok(UserDto {
                    discord_id: user_model.discord_id.parse::<u64>().map_err(|e| {
                        AppError::InternalError(format!("Failed to parse discord_id: {}", e))
                    })?,
                    name: user_model.name,
                    admin: user_model.admin,
                })
            })
            .collect::<Result<Vec<UserDto>, AppError>>()?;

        Ok(users)
    }

    pub async fn add_admin(&self, user_id: u64) -> Result<(), AppError> {
        let user_repo = UserRepository::new(self.db);

        // Verify user exists
        let user = user_repo.find_by_discord_id(user_id).await?;
        if user.is_none() {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        user_repo.set_admin(user_id, true).await?;

        Ok(())
    }

    pub async fn remove_admin(&self, user_id: u64) -> Result<(), AppError> {
        let user_repo = UserRepository::new(self.db);

        // Verify user exists
        let user = user_repo.find_by_discord_id(user_id).await?;
        if user.is_none() {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        user_repo.set_admin(user_id, false).await?;

        Ok(())
    }

    /// Gets all guilds for a specific user
    ///
    /// Retrieves all Discord guilds (timerboards) that the user is a member of.
    /// If the user is an admin, returns all guilds in the system.
    ///
    /// # Arguments
    /// - `user_id`: Discord ID of the user (u64)
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuildDto>)`: Vector of guilds the user has access to
    /// - `Err(AppError)`: Database error or parse error
    pub async fn get_user_guilds(&self, user_id: u64) -> Result<Vec<DiscordGuildDto>, AppError> {
        let user_repo = UserRepository::new(self.db);
        let guild_repo = DiscordGuildRepository::new(self.db);

        // Check if user is admin
        let user = user_repo.find_by_discord_id(user_id).await?;
        let is_admin = user.map(|u| u.admin).unwrap_or(false);

        // If admin, return all guilds; otherwise return only user's guilds
        let guild_models = if is_admin {
            guild_repo.get_all().await?
        } else {
            guild_repo.get_guilds_for_user(user_id).await?
        };

        let guilds: Vec<DiscordGuildDto> = guild_models
            .into_iter()
            .map(|guild_model| DiscordGuildDto {
                guild_id: guild_model.guild_id,
                name: guild_model.name,
                icon_hash: guild_model.icon_hash,
            })
            .collect();

        Ok(guilds)
    }
}
