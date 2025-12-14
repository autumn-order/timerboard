use chrono::Utc;
use migration::OnConflict;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
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

        // Convert u64 to i32 - Note: This may overflow for very large Discord IDs
        // TODO: Migrate user.discord_id to String type like other Discord ID fields
        let discord_id_i32 = user.id.get() as i32;

        entity::prelude::User::insert(entity::user::ActiveModel {
            discord_id: ActiveValue::Set(discord_id_i32),
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

    pub async fn find_by_id(&self, user_id: i32) -> Result<Option<entity::user::Model>, DbErr> {
        entity::prelude::User::find_by_id(user_id)
            .one(self.db)
            .await
    }

    /// Finds a user by their Discord ID
    ///
    /// Searches for a user in the database using their Discord-assigned user ID.
    /// Used to check if a Discord user has logged into the application.
    ///
    /// # Arguments
    /// - `discord_id`: Discord's unique identifier for the user (u64)
    ///
    /// # Returns
    /// - `Ok(Some(Model))`: User found in database (has logged in)
    /// - `Ok(None)`: User not found (has not logged into the app)
    /// - `Err(DbErr)`: Database error during query
    pub async fn find_by_discord_id(
        &self,
        discord_id: u64,
    ) -> Result<Option<entity::user::Model>, DbErr> {
        // Convert u64 to i32 - Note: This may overflow for very large Discord IDs
        // TODO: Migrate user.discord_id to String type like other Discord ID fields
        let discord_id_i32 = discord_id as i32;

        entity::prelude::User::find()
            .filter(entity::user::Column::DiscordId.eq(discord_id_i32))
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

    /// Updates the last guild sync timestamp to current time
    ///
    /// Sets the last_guild_sync_at column to the current UTC timestamp.
    /// Used after successfully syncing a user's guild memberships.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    ///
    /// # Returns
    /// - `Ok(())`: Timestamp updated successfully
    /// - `Err(DbErr)`: Database error during update
    pub async fn update_guild_sync_timestamp(&self, user_id: i32) -> Result<(), DbErr> {
        entity::prelude::User::update_many()
            .filter(entity::user::Column::Id.eq(user_id))
            .col_expr(
                entity::user::Column::LastGuildSyncAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Updates the last role sync timestamp to current time
    ///
    /// Sets the last_role_sync_at column to the current UTC timestamp.
    /// Used after successfully syncing a user's role memberships.
    ///
    /// # Arguments
    /// - `user_id`: Database ID of the user
    ///
    /// # Returns
    /// - `Ok(())`: Timestamp updated successfully
    /// - `Err(DbErr)`: Database error during update
    pub async fn update_role_sync_timestamp(&self, user_id: i32) -> Result<(), DbErr> {
        entity::prelude::User::update_many()
            .filter(entity::user::Column::Id.eq(user_id))
            .col_expr(
                entity::user::Column::LastRoleSyncAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Updates the last guild sync timestamp for multiple users at once
    ///
    /// Sets the last_guild_sync_at column to the current UTC timestamp for all specified users.
    /// Used after successfully syncing guild memberships for multiple users during bot startup.
    ///
    /// # Arguments
    /// - `user_ids`: Slice of database IDs of users to update
    ///
    /// # Returns
    /// - `Ok(())`: Timestamps updated successfully
    /// - `Err(DbErr)`: Database error during update
    pub async fn update_guild_sync_timestamps(&self, user_ids: &[i32]) -> Result<(), DbErr> {
        if user_ids.is_empty() {
            return Ok(());
        }

        entity::prelude::User::update_many()
            .filter(entity::user::Column::Id.is_in(user_ids.iter().copied()))
            .col_expr(
                entity::user::Column::LastGuildSyncAt,
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
    /// - `user_ids`: Slice of database IDs of users to update
    ///
    /// # Returns
    /// - `Ok(())`: Timestamps updated successfully
    /// - `Err(DbErr)`: Database error during update
    pub async fn update_role_sync_timestamps(&self, user_ids: &[i32]) -> Result<(), DbErr> {
        if user_ids.is_empty() {
            return Ok(());
        }

        entity::prelude::User::update_many()
            .filter(entity::user::Column::Id.is_in(user_ids.iter().copied()))
            .col_expr(
                entity::user::Column::LastRoleSyncAt,
                sea_orm::sea_query::Expr::value(Utc::now().naive_utc()),
            )
            .exec(self.db)
            .await?;
        Ok(())
    }
}
