use chrono::Utc;
use migration::OnConflict;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serenity::all::User as DiscordUser;

pub struct UserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(
        &self,
        user: DiscordUser,
        is_admin: Option<bool>,
    ) -> Result<entity::user::Model, DbErr> {
        // Build list of columns to update on conflict
        let mut update_columns = vec![entity::user::Column::Name];

        // Only update admin column if is_admin is Some
        if is_admin.is_some() {
            update_columns.push(entity::user::Column::Admin);
        }

        entity::prelude::User::insert(entity::user::ActiveModel {
            discord_id: ActiveValue::Set(user.id.get().to_string()),
            name: ActiveValue::Set(user.name),
            admin: ActiveValue::Set(is_admin.unwrap_or(false)),
            ..Default::default()
        })
        // Update user name in case it may have changed since last login
        // Only update admin if is_admin is Some to prevent resetting admin status
        .on_conflict(
            OnConflict::column(entity::user::Column::DiscordId)
                .update_columns(update_columns)
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await
    }

    pub async fn find_by_discord_id(
        &self,
        user_id: u64,
    ) -> Result<Option<entity::user::Model>, DbErr> {
        entity::prelude::User::find_by_id(user_id.to_string())
            .one(self.db)
            .await
    }

    /// Checks if any admin users exist in the database.
    ///
    /// # Returns
    /// - `Ok(true)` if at least one admin user exists
    /// - `Ok(false)` if no admin users exist
    /// - `Err(DbErr)` if the database query fails
    pub async fn admin_exists(&self) -> Result<bool, DbErr> {
        let admin_count = entity::prelude::User::find()
            .filter(entity::user::Column::Admin.eq(true))
            .count(self.db)
            .await?;

        Ok(admin_count > 0)
    }

    /// Updates the last role sync timestamp to current time
    ///
    /// Sets the last_role_sync_at column to the current UTC timestamp.
    /// Used after successfully syncing a user's role memberships.
    ///
    /// # Arguments
    /// - `user_id`: Discord ID of the user (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Timestamp updated successfully
    /// - `Err(DbErr)`: Database error during update
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

    /// Updates the last role sync timestamp for multiple users at once
    ///
    /// Sets the last_role_sync_at column to the current UTC timestamp for all specified users.
    /// Used after successfully syncing role memberships for multiple users during bot startup.
    ///
    /// # Arguments
    /// - `user_ids`: Slice of Discord IDs of users to update (u64)
    ///
    /// # Returns
    /// - `Ok(())`: Timestamps updated successfully
    /// - `Err(DbErr)`: Database error during update
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

    /// Gets all users with pagination
    ///
    /// Returns a paginated list of all users in the application, ordered by name.
    ///
    /// # Arguments
    /// - `page`: Zero-indexed page number
    /// - `per_page`: Number of items per page
    ///
    /// # Returns
    /// - `Ok((users, total))`: Tuple of user models and total count
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_all_paginated(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<entity::user::Model>, u64), DbErr> {
        let paginator = entity::prelude::User::find()
            .order_by_asc(entity::user::Column::Name)
            .paginate(self.db, per_page);

        let total = paginator.num_pages().await?;
        let users = paginator.fetch_page(page).await?;

        Ok((users, total))
    }

    /// Gets all admin users
    ///
    /// Returns a list of all users with admin privileges, ordered by name.
    ///
    /// # Returns
    /// - `Ok(users)`: Vector of admin user models
    /// - `Err(DbErr)`: Database error during query
    pub async fn get_all_admins(&self) -> Result<Vec<entity::user::Model>, DbErr> {
        entity::prelude::User::find()
            .filter(entity::user::Column::Admin.eq(true))
            .order_by_asc(entity::user::Column::Name)
            .all(self.db)
            .await
    }

    /// Sets admin status for a user
    ///
    /// Updates the admin column for the specified user.
    ///
    /// # Arguments
    /// - `user_id`: Discord ID of the user (u64)
    /// - `is_admin`: Whether the user should be an admin
    ///
    /// # Returns
    /// - `Ok(())`: Admin status updated successfully
    /// - `Err(DbErr)`: Database error during update
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
