//! User service for business logic.
//!
//! This module provides the `UserService` for managing user-related business logic.
//! It orchestrates user queries, admin management, and guild access control while
//! working with domain models rather than DTOs.

use sea_orm::DatabaseConnection;

use crate::server::{
    data::{discord::guild::DiscordGuildRepository, user::UserRepository},
    error::AppError,
    model::{
        discord::DiscordGuild,
        user::{GetAllUsersParam, GetUserParam, PaginatedUsers, SetAdminParam, User},
    },
};

/// Service providing business logic for user management.
///
/// This struct holds a reference to the database connection and provides methods
/// for user queries, admin management, and guild access control.
pub struct UserService<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> UserService<'a> {
    /// Creates a new UserService instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `UserService` - New service instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Retrieves a user by their Discord ID.
    ///
    /// Queries the database for a user with the specified Discord ID and returns
    /// their full domain model if found.
    ///
    /// # Arguments
    /// - `param` - Parameters containing the Discord user ID to look up
    ///
    /// # Returns
    /// - `Ok(Some(User))` - User found with full domain data
    /// - `Ok(None)` - No user found with that Discord ID
    /// - `Err(AppError::Database)` - Database error during query
    pub async fn get_user(&self, param: GetUserParam) -> Result<Option<User>, AppError> {
        let user_repo = UserRepository::new(self.db);
        let user = user_repo.find_by_id(param.discord_id).await?;
        Ok(user)
    }

    /// Retrieves all users with pagination.
    ///
    /// Returns a paginated collection of users ordered alphabetically by name.
    /// Calculates total pages based on the per_page parameter and total user count.
    ///
    /// # Arguments
    /// - `param` - Parameters specifying page number and users per page
    ///
    /// # Returns
    /// - `Ok(PaginatedUsers)` - Users for the requested page with pagination metadata
    /// - `Err(AppError::Database)` - Database error during pagination query
    pub async fn get_all_users(&self, param: GetAllUsersParam) -> Result<PaginatedUsers, AppError> {
        let user_repo = UserRepository::new(self.db);

        let (users, total_items) = user_repo
            .get_all_paginated(param.page, param.per_page)
            .await?;

        let total_pages = (total_items as f64 / param.per_page as f64).ceil() as u64;

        Ok(PaginatedUsers {
            users,
            total: total_items,
            page: param.page,
            per_page: param.per_page,
            total_pages,
        })
    }

    /// Retrieves all users with admin privileges.
    ///
    /// Returns a list of all users who have admin status, ordered alphabetically by name.
    /// Used for displaying admin lists and managing administrative access.
    ///
    /// # Returns
    /// - `Ok(Vec<User>)` - Vector of all admin users (empty if no admins exist)
    /// - `Err(AppError::Database)` - Database error during query
    pub async fn get_all_admins(&self) -> Result<Vec<User>, AppError> {
        let user_repo = UserRepository::new(self.db);
        let admins = user_repo.get_all_admins().await?;
        Ok(admins)
    }

    /// Grants admin privileges to a user.
    ///
    /// Verifies the user exists in the database before setting their admin status to true.
    /// Returns an error if the user is not found.
    ///
    /// # Arguments
    /// - `param` - Parameters containing the Discord user ID and admin status (should be true)
    ///
    /// # Returns
    /// - `Ok(())` - Admin status successfully granted
    /// - `Err(AppError::NotFound)` - User with specified Discord ID does not exist
    /// - `Err(AppError::Database)` - Database error during query or update
    pub async fn add_admin(&self, param: SetAdminParam) -> Result<(), AppError> {
        let user_repo = UserRepository::new(self.db);

        // Verify user exists
        let user = user_repo.find_by_id(param.discord_id).await?;
        if user.is_none() {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        user_repo
            .set_admin(param.discord_id, param.is_admin)
            .await?;

        Ok(())
    }

    /// Revokes admin privileges from a user.
    ///
    /// Verifies the user exists in the database before setting their admin status to false.
    /// Returns an error if the user is not found.
    ///
    /// # Arguments
    /// - `param` - Parameters containing the Discord user ID and admin status (should be false)
    ///
    /// # Returns
    /// - `Ok(())` - Admin status successfully revoked
    /// - `Err(AppError::NotFound)` - User with specified Discord ID does not exist
    /// - `Err(AppError::Database)` - Database error during query or update
    pub async fn remove_admin(&self, param: SetAdminParam) -> Result<(), AppError> {
        let user_repo = UserRepository::new(self.db);

        // Verify user exists
        let user = user_repo.find_by_id(param.discord_id).await?;
        if user.is_none() {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        user_repo
            .set_admin(param.discord_id, param.is_admin)
            .await?;

        Ok(())
    }

    /// Retrieves all guilds accessible to a user.
    ///
    /// Returns all Discord guilds (timerboards) that the user has access to based on their
    /// permissions. Admin users receive all guilds in the system, while regular users only
    /// receive guilds they are members of.
    ///
    /// # Arguments
    /// - `param` - Parameters containing the Discord user ID to check access for
    ///
    /// # Returns
    /// - `Ok(Vec<DiscordGuild>)` - Vector of guilds the user has access to
    /// - `Err(AppError::Database)` - Database error during query
    /// - `Err(AppError::Custom)` - Error parsing guild_id from database
    pub async fn get_user_guilds(
        &self,
        param: GetUserParam,
    ) -> Result<Vec<DiscordGuild>, AppError> {
        let user_repo = UserRepository::new(self.db);
        let guild_repo = DiscordGuildRepository::new(self.db);

        // Check if user is admin
        let user = user_repo.find_by_id(param.discord_id).await?;
        let is_admin = user.map(|u| u.admin).unwrap_or(false);

        // If admin, return all guilds; otherwise return only user's guilds
        let guilds = if is_admin {
            guild_repo.get_all().await?
        } else {
            guild_repo.get_guilds_for_user(param.discord_id).await?
        };

        Ok(guilds)
    }
}
