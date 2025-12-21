//! User category permission repository for fleet category access control.
//!
//! This module provides the `UserCategoryPermissionRepository` for checking and filtering
//! fleet categories based on user Discord role permissions. It handles all permission-related
//! queries, separating permission logic from category data operations for better maintainability
//! and single responsibility.

use crate::server::model::category::FleetCategoryListItem;
use sea_orm::{
    sea_query::Condition, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};

/// Repository for user category permission operations.
///
/// Handles checking and filtering categories based on user Discord role permissions.
/// All methods in this repository assume the caller has already verified the user exists
/// and belongs to the specified guild. Admin checks should be performed before calling
/// these methods - admins typically bypass permission checks entirely.
///
/// # Example
///
/// ```rust,ignore
/// use server::data::user_category_permission::UserCategoryPermissionRepository;
///
/// let repo = UserCategoryPermissionRepository::new(&db);
///
/// // Check if user can view a specific category
/// let can_view = repo.user_can_view_category(user_id, guild_id, category_id).await?;
///
/// // Get all category IDs the user can create fleets in
/// let creatable_ids = repo.get_creatable_category_ids_by_user(user_id, guild_id).await?;
/// ```
pub struct UserCategoryPermissionRepository<'a> {
    /// Database connection for executing queries.
    db: &'a DatabaseConnection,
}

impl<'a> UserCategoryPermissionRepository<'a> {
    /// Creates a new UserCategoryPermissionRepository.
    ///
    /// # Arguments
    /// - `db` - Database connection reference
    ///
    /// # Returns
    /// - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Gets fleet categories that a user can create or manage.
    ///
    /// Returns categories where the user has can_create OR can_manage permission
    /// through their Discord roles. Admins are not handled here - check admin status
    /// before calling this method to grant full access.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<FleetCategoryListItem>)` - Categories the user can create/manage
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_manageable_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<FleetCategoryListItem>, DbErr> {
        use sea_orm::Condition;

        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_create or can_manage permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(
                Condition::any()
                    .add(entity::fleet_category_access_role::Column::CanCreate.eq(true))
                    .add(entity::fleet_category_access_role::Column::CanManage.eq(true)),
            )
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Get the actual category models for this guild
        let categories = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .order_by_asc(entity::fleet_category::Column::Name)
            .all(self.db)
            .await?;

        categories
            .into_iter()
            .map(FleetCategoryListItem::from_entity)
            .collect()
    }

    /// Gets fleet category IDs that a user can view.
    ///
    /// Returns category IDs where the user has can_view permission through their
    /// Discord roles. Used for filtering fleet lists and category dropdowns.
    /// Admins are not handled here - check admin status before calling this method
    /// to grant access to all categories.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<i32>)` - Category IDs the user can view
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_viewable_category_ids_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<i32>, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_view permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanView.eq(true))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Verify these categories belong to the specified guild
        let guild_category_ids: Vec<i32> = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        Ok(guild_category_ids)
    }

    /// Gets fleet category IDs that a user can create fleets in.
    ///
    /// Returns category IDs where the user has can_create permission through their
    /// Discord roles. Used for filtering category options when creating new fleets.
    /// Admins are not handled here - check admin status before calling this method
    /// to grant access to all categories.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<i32>)` - Category IDs the user can create fleets in
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_creatable_category_ids_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<i32>, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_create permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanCreate.eq(true))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Verify these categories belong to the specified guild
        let guild_category_ids: Vec<i32> = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        Ok(guild_category_ids)
    }

    /// Gets fleet category IDs that a user can manage.
    ///
    /// Returns category IDs where the user has can_manage permission through their
    /// Discord roles. Used for filtering categories in management interfaces.
    /// Admins are not handled here - check admin status before calling this method
    /// to grant access to all categories.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Ok(Vec<i32>)` - Category IDs the user can manage
    /// - `Err(DbErr)` - Database error during query
    pub async fn get_manageable_category_ids_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Vec<i32>, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find all category IDs where the user has can_manage permission
        let category_ids: Vec<i32> = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanManage.eq(true))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.fleet_category_id)
            .collect();

        if category_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Verify these categories belong to the specified guild
        let guild_category_ids: Vec<i32> = entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::GuildId.eq(guild_id.to_string()))
            .filter(entity::fleet_category::Column::Id.is_in(category_ids))
            .all(self.db)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        Ok(guild_category_ids)
    }

    /// Checks if a user has view access to a specific category.
    ///
    /// Verifies that at least one of the user's Discord roles has can_view permission
    /// for the specified category. Used for authorization checks before displaying
    /// category data or fleets within a category.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `_guild_id` - Discord guild ID (currently unused but kept for API consistency)
    /// - `category_id` - Fleet category ID to check access for
    ///
    /// # Returns
    /// - `Ok(true)` - User has view access to the category
    /// - `Ok(false)` - User does not have view access
    /// - `Err(DbErr)` - Database error during query
    pub async fn user_can_view_category(
        &self,
        user_id: u64,
        category_id: i32,
    ) -> Result<bool, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(false);
        }

        // Check if any of the user's roles have view access to this category
        let access_count = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category_id))
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanView.eq(true))
            .count(self.db)
            .await?;

        Ok(access_count > 0)
    }

    /// Checks if a user has create access to a specific category.
    ///
    /// Verifies that at least one of the user's Discord roles has can_create or can_manage
    /// permission for the specified category. Manage permission implicitly grants create access.
    /// Used for authorization checks before allowing fleet creation in a category.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `_guild_id` - Discord guild ID (currently unused but kept for API consistency)
    /// - `category_id` - Fleet category ID to check access for
    ///
    /// # Returns
    /// - `Ok(true)` - User has create or manage access to the category
    /// - `Ok(false)` - User does not have create or manage access
    /// - `Err(DbErr)` - Database error during query
    pub async fn user_can_create_category(
        &self,
        user_id: u64,
        category_id: i32,
    ) -> Result<bool, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(false);
        }

        // Check if any of the user's roles have create or manage access to this category
        // Manage permission implicitly grants create access
        let access_count = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category_id))
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(
                Condition::any()
                    .add(entity::fleet_category_access_role::Column::CanCreate.eq(true))
                    .add(entity::fleet_category_access_role::Column::CanManage.eq(true)),
            )
            .count(self.db)
            .await?;

        Ok(access_count > 0)
    }

    /// Checks if a user has manage access to a specific category.
    ///
    /// Verifies that at least one of the user's Discord roles has can_manage permission
    /// for the specified category. Used for authorization checks before allowing
    /// category updates, deletion, or other administrative operations.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    /// - `_guild_id` - Discord guild ID (currently unused but kept for API consistency)
    /// - `category_id` - Fleet category ID to check access for
    ///
    /// # Returns
    /// - `Ok(true)` - User has manage access to the category
    /// - `Ok(false)` - User does not have manage access
    /// - `Err(DbErr)` - Database error during query
    pub async fn user_can_manage_category(
        &self,
        user_id: u64,
        category_id: i32,
    ) -> Result<bool, DbErr> {
        // First, get all role IDs that the user has in this guild
        let user_role_ids: Vec<String> = entity::prelude::UserDiscordGuildRole::find()
            .filter(entity::user_discord_guild_role::Column::UserId.eq(user_id.to_string()))
            .all(self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        if user_role_ids.is_empty() {
            return Ok(false);
        }

        // Check if any of the user's roles have manage access to this category
        let access_count = entity::prelude::FleetCategoryAccessRole::find()
            .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category_id))
            .filter(entity::fleet_category_access_role::Column::RoleId.is_in(user_role_ids))
            .filter(entity::fleet_category_access_role::Column::CanManage.eq(true))
            .count(self.db)
            .await?;

        Ok(access_count > 0)
    }
}
