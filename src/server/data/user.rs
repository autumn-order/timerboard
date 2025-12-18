//! User data repository for database operations.
//!
//! This module provides the `UserRepository` for managing user records in the database.
//! It handles user creation, updates, queries, and admin status management with proper
//! conversion between entity models and parameter models at the infrastructure boundary.

use crate::server::model::user::{UpsertUserParam, User};
use chrono::Utc;
use migration::OnConflict;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};

/// Repository providing database operations for user management.
///
/// This struct holds a reference to the database connection and provides methods
/// for creating, reading, updating, and querying user records.
pub struct UserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    /// Creates a new UserRepository instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `UserRepository` - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Upserts a user from parameter model.
    ///
    /// Inserts a new user or updates an existing user's name and optionally their admin status.
    /// The admin status is only updated if explicitly provided (Some value), preventing
    /// accidental removal of admin privileges during regular login operations.
    ///
    /// # Arguments
    /// - `param` - User upsert parameters including discord_id, name, and optional admin status
    ///
    /// # Returns
    /// - `Ok(User)` - The created or updated user
    /// - `Err(DbErr)` - Database error during insert or update
    pub async fn upsert(&self, param: UpsertUserParam) -> Result<User, DbErr> {
        // Build list of columns to update on conflict
        let mut update_columns = vec![entity::user::Column::Name];

        // Only update admin column if is_admin is Some
        if param.is_admin.is_some() {
            update_columns.push(entity::user::Column::Admin);
        }

        let entity = entity::prelude::User::insert(entity::user::ActiveModel {
            discord_id: ActiveValue::Set(param.discord_id),
            name: ActiveValue::Set(param.name),
            admin: ActiveValue::Set(param.is_admin.unwrap_or(false)),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::column(entity::user::Column::DiscordId)
                .update_columns(update_columns)
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await?;

        Ok(User::from_entity(entity))
    }

    /// Finds a user by their Discord ID.
    ///
    /// Queries the database for a user with the specified Discord ID and returns
    /// their full information if found.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID as u64
    ///
    /// # Returns
    /// - `Ok(Some(User))` - User found with full data
    /// - `Ok(None)` - No user found with that Discord ID
    /// - `Err(DbErr)` - Database error during query
    pub async fn find_by_discord_id(&self, user_id: u64) -> Result<Option<User>, DbErr> {
        let entity = entity::prelude::User::find_by_id(user_id.to_string())
            .one(self.db)
            .await?;

        Ok(entity.map(User::from_entity))
    }

    /// Checks if any admin users exist in the database.
    ///
    /// Performs a count query filtered by admin status to determine if the application
    /// has at least one admin user. Used during first-time setup to determine if the
    /// first user should be automatically granted admin privileges.
    ///
    /// # Returns
    /// - `Ok(true)` - At least one admin user exists in the database
    /// - `Ok(false)` - No admin users exist (first-time setup scenario)
    /// - `Err(DbErr)` - Database error during count query
    pub async fn admin_exists(&self) -> Result<bool, DbErr> {
        let admin_count = entity::prelude::User::find()
            .filter(entity::user::Column::Admin.eq(true))
            .count(self.db)
            .await?;

        Ok(admin_count > 0)
    }

    /// Updates the last role sync timestamp for a single user.
    ///
    /// Sets the last_role_sync_at column to the current UTC timestamp for the specified user.
    /// Used after successfully syncing a user's role memberships with Discord to track when
    /// the sync last occurred.
    ///
    /// # Arguments
    /// - `user_id` - Discord ID of the user as u64
    ///
    /// # Returns
    /// - `Ok(())` - Timestamp updated successfully (or no matching user found)
    /// - `Err(DbErr)` - Database error during update operation
    pub async fn update_role_sync_timestamp(&self, user_id: u64) -> Result<(), DbErr> {
        entity::prelude::User::update_many()
            .filter(entity::user::Column::DiscordId.eq(user_id.to_string()))
            .col_expr(
                entity::user::Column::LastRoleSyncAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Updates the last role sync timestamp for multiple users at once.
    ///
    /// Sets the last_role_sync_at column to the current UTC timestamp for all specified users
    /// in a single database operation. Used after successfully syncing role memberships for
    /// multiple users during bot startup or batch sync operations.
    ///
    /// # Arguments
    /// - `user_ids` - Slice of Discord user IDs as u64
    ///
    /// # Returns
    /// - `Ok(())` - Timestamps updated successfully (returns early if slice is empty)
    /// - `Err(DbErr)` - Database error during batch update operation
    pub async fn update_role_sync_timestamps(&self, user_ids: &[u64]) -> Result<(), DbErr> {
        if user_ids.is_empty() {
            return Ok(());
        }

        let user_id_strings: Vec<String> = user_ids.iter().map(|id| id.to_string()).collect();
        entity::prelude::User::update_many()
            .filter(entity::user::Column::DiscordId.is_in(user_id_strings))
            .col_expr(
                entity::user::Column::LastRoleSyncAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Gets all users with pagination.
    ///
    /// Returns a paginated list of all users in the application, ordered alphabetically by name.
    /// Used for admin user management interfaces to display and manage the user base.
    ///
    /// # Arguments
    /// - `page` - Zero-indexed page number
    /// - `per_page` - Number of users to return per page
    ///
    /// # Returns
    /// - `Ok((users, total))` - Vector of users for the requested page and total user count
    /// - `Err(DbErr)` - Database error during pagination query
    pub async fn get_all_paginated(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<User>, u64), DbErr> {
        let paginator = entity::prelude::User::find()
            .order_by_asc(entity::user::Column::Name)
            .paginate(self.db, per_page);

        let total = paginator.num_pages().await?;
        let entities = paginator.fetch_page(page).await?;
        let users = entities.into_iter().map(User::from_entity).collect();

        Ok((users, total))
    }

    /// Gets all admin users.
    ///
    /// Returns a list of all users with admin privileges, ordered alphabetically by name.
    /// Used for displaying admin lists and checking admin permissions.
    ///
    /// # Returns
    /// - `Ok(Vec<User>)` - Vector of all admin users (empty if no admins exist)
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_all_admins(&self) -> Result<Vec<User>, DbErr> {
        let entities = entity::prelude::User::find()
            .filter(entity::user::Column::Admin.eq(true))
            .order_by_asc(entity::user::Column::Name)
            .all(self.db)
            .await?;

        Ok(entities.into_iter().map(User::from_entity).collect())
    }

    /// Sets admin status for a user.
    ///
    /// Updates the admin column for the specified user to grant or revoke admin privileges.
    /// Used by admin management endpoints to control which users have elevated permissions.
    ///
    /// # Arguments
    /// - `user_id` - Discord ID of the user as u64
    /// - `is_admin` - Whether the user should have admin privileges
    ///
    /// # Returns
    /// - `Ok(())` - Admin status updated successfully (or no matching user found)
    /// - `Err(DbErr)` - Database error during update operation
    pub async fn set_admin(&self, user_id: u64, is_admin: bool) -> Result<(), DbErr> {
        entity::prelude::User::update_many()
            .filter(entity::user::Column::DiscordId.eq(user_id.to_string()))
            .col_expr(
                entity::user::Column::Admin,
                sea_orm::sea_query::Expr::value(is_admin),
            )
            .exec(self.db)
            .await?;
        Ok(())
    }
}
