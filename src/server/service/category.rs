use sea_orm::DatabaseConnection;

use crate::server::{
    data::{
        category::FleetCategoryRepository,
        user_category_permission::UserCategoryPermissionRepository,
    },
    error::AppError,
    model::category::{
        CreateFleetCategoryParams, FleetCategory, FleetCategoryListItem, PaginatedFleetCategories,
        UpdateFleetCategoryParams,
    },
};

/// Maximum number of categories to fetch for admin users in a single query.
const MAX_ADMIN_CATEGORIES: u64 = 1000;

/// Service for fleet category business logic.
///
/// Provides methods for managing fleet categories including creation, updates,
/// deletion, and permission-based queries. Acts as an orchestration layer between
/// controllers and the data repository.
pub struct FleetCategoryService<'a> {
    /// Database connection for repository operations.
    db: &'a DatabaseConnection,
}

impl<'a> FleetCategoryService<'a> {
    /// Creates a new FleetCategoryService instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `FleetCategoryService` - New service instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet category for a guild.
    ///
    /// Creates a fleet category with the provided parameters and returns the full
    /// category data including all related entities (ping formats, roles, channels).
    /// After creation, fetches the complete category with all relations to return.
    ///
    /// # Arguments
    /// - `params` - Category creation parameters including guild_id, name, and duration fields
    ///
    /// # Returns
    /// - `Ok(FleetCategory)` - Created category with all relations loaded
    /// - `Err(AppError::Database)` - Database error during creation or fetch
    /// - `Err(AppError::NotFound)` - Category not found after creation (should not occur)
    /// - `Err(AppError::Conversion)` - Error converting entity to domain model
    pub async fn create(
        &self,
        params: CreateFleetCategoryParams,
    ) -> Result<FleetCategory, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let category = repo.create(params).await?;

        // Fetch full category with relations
        let full_result = repo
            .find_by_id(category.id)
            .await?
            .ok_or_else(|| AppError::NotFound("Category not found after creation".to_string()))?;

        Ok(FleetCategory::from_with_relations(full_result)?)
    }

    /// Gets a specific fleet category by ID with all related data.
    ///
    /// Retrieves a fleet category including all related entities such as ping formats,
    /// access roles, ping roles, and channels. Returns None if the category doesn't exist.
    ///
    /// # Arguments
    /// - `id` - Category ID to retrieve
    ///
    /// # Returns
    /// - `Ok(Some(FleetCategory))` - Category found with all relations
    /// - `Ok(None)` - Category not found
    /// - `Err(AppError::Database)` - Database error during fetch
    /// - `Err(AppError::Conversion)` - Error converting entity to domain model
    pub async fn get_by_id(&self, id: i32) -> Result<Option<FleetCategory>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let result = repo.find_by_id(id).await?;

        result
            .map(FleetCategory::from_with_relations)
            .transpose()
            .map_err(Into::into)
    }

    /// Gets paginated fleet categories for a guild with counts.
    ///
    /// Retrieves a paginated list of categories for a guild, including counts of
    /// related entities (ping formats, roles, channels) for each category. Calculates
    /// total pages based on the total count and per_page value.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID to filter categories
    /// - `page` - Zero-based page number
    /// - `per_page` - Number of categories per page
    ///
    /// # Returns
    /// - `Ok(PaginatedFleetCategories)` - Paginated results with categories and metadata
    /// - `Err(AppError::Database)` - Database error during fetch
    /// - `Err(AppError::Conversion)` - Error converting entity to domain model
    pub async fn get_paginated(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedFleetCategories, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let (categories, total) = repo
            .get_by_guild_id_paginated(guild_id, page, per_page)
            .await?;

        let total_pages = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as u64
        } else {
            0
        };

        let categories: Result<Vec<_>, _> = categories
            .into_iter()
            .map(FleetCategoryListItem::from_with_counts)
            .collect();

        Ok(PaginatedFleetCategories {
            categories: categories?,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Updates a fleet category's name and duration fields.
    ///
    /// Updates the specified fields of a category and returns the updated category
    /// with all relations. Validates that the category exists and belongs to the
    /// specified guild before updating. Returns None if validation fails.
    ///
    /// # Arguments
    /// - `params` - Update parameters including id, guild_id, and fields to update
    ///
    /// # Returns
    /// - `Ok(Some(FleetCategory))` - Category updated successfully with all relations
    /// - `Ok(None)` - Category doesn't exist or doesn't belong to the guild
    /// - `Err(AppError::Database)` - Database error during update or fetch
    /// - `Err(AppError::Conversion)` - Error converting entity to domain model
    pub async fn update(
        &self,
        params: UpdateFleetCategoryParams,
    ) -> Result<Option<FleetCategory>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Check if category exists and belongs to the guild
        if !repo.exists_in_guild(params.id, params.guild_id).await? {
            return Ok(None);
        }

        let _category = repo.update(params.clone()).await?;

        // Fetch full category with relations
        let full_result = repo.find_by_id(params.id).await?;

        full_result
            .map(FleetCategory::from_with_relations)
            .transpose()
            .map_err(Into::into)
    }

    /// Deletes a fleet category.
    ///
    /// Deletes the specified category after verifying it exists and belongs to the
    /// specified guild. Related entities are handled according to database constraints
    /// (cascading deletes or restrictions based on foreign key setup).
    ///
    /// # Arguments
    /// - `id` - Category ID to delete
    /// - `guild_id` - Discord guild ID for ownership validation
    ///
    /// # Returns
    /// - `Ok(true)` - Category deleted successfully
    /// - `Ok(false)` - Category not found or doesn't belong to guild
    /// - `Err(AppError::Database)` - Database error during deletion or foreign key constraint violation
    pub async fn delete(&self, id: i32, guild_id: u64) -> Result<bool, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Check if category exists and belongs to the guild
        if !repo.exists_in_guild(id, guild_id).await? {
            return Ok(false);
        }

        repo.delete(id).await?;

        Ok(true)
    }

    /// Gets fleet categories by ping format ID.
    ///
    /// Retrieves all categories associated with a specific ping format, including
    /// counts of related entities for each category. Returns an empty vector if no
    /// categories are found for the ping format.
    ///
    /// # Arguments
    /// - `ping_format_id` - Ping format ID to filter categories
    ///
    /// # Returns
    /// - `Ok(Vec<FleetCategoryListItem>)` - Vector of categories with counts (may be empty)
    /// - `Err(AppError::Database)` - Database error during fetch
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<FleetCategoryListItem>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let categories = repo.get_by_ping_format_id(ping_format_id).await?;

        Ok(categories)
    }

    /// Gets fleet categories that a user can create or manage.
    ///
    /// Returns categories where the user has can_create OR can_manage permission
    /// through their roles. Admins bypass permission checks and get all categories
    /// for the guild. Returns an empty vector if the user has no manageable categories.
    ///
    /// # Arguments
    /// - `user_id` - Discord ID of the user
    /// - `guild_id` - Discord guild ID to filter categories
    /// - `is_admin` - Whether the user is an admin (bypasses permission checks)
    ///
    /// # Returns
    /// - `Ok(Vec<FleetCategoryListItem>)` - Vector of manageable categories with counts (may be empty)
    /// - `Err(AppError::Database)` - Database error during fetch
    /// - `Err(AppError::Conversion)` - Error converting entity to domain model
    pub async fn get_manageable_by_user(
        &self,
        user_id: u64,
        guild_id: u64,
        is_admin: bool,
    ) -> Result<Vec<FleetCategoryListItem>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let categories = if is_admin {
            // Admins get all categories for the guild
            let (cats, _) = repo
                .get_by_guild_id_paginated(guild_id, 0, MAX_ADMIN_CATEGORIES)
                .await?;
            cats.into_iter()
                .map(FleetCategoryListItem::from_with_counts)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            // Regular users get only categories they can manage
            let permission_repo = UserCategoryPermissionRepository::new(self.db);
            permission_repo
                .get_manageable_by_user(user_id, guild_id)
                .await?
        };

        Ok(categories)
    }
}
