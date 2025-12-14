use sea_orm::DatabaseConnection;

use crate::server::{
    data::category::FleetCategoryRepository,
    error::AppError,
    model::category::{
        CreateFleetCategoryParams, FleetCategory, FleetCategoryListItem, PaginatedFleetCategories,
        UpdateFleetCategoryParams,
    },
};

pub struct FleetCategoryService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetCategoryService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet category for a guild
    pub async fn create(
        &self,
        params: CreateFleetCategoryParams,
    ) -> Result<FleetCategory, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let result = repo.create(params).await?;

        // Fetch full category with relations
        let full_result = repo
            .get_by_id(result.category.id)
            .await?
            .ok_or_else(|| AppError::NotFound("Category not found after creation".to_string()))?;

        Ok(FleetCategory::from_with_relations(full_result)?)
    }

    /// Gets a specific fleet category by ID with all related data
    pub async fn get_by_id(&self, id: i32) -> Result<Option<FleetCategory>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let result = repo.get_by_id(id).await?;

        result
            .map(FleetCategory::from_with_relations)
            .transpose()
            .map_err(Into::into)
    }

    /// Gets paginated fleet categories for a guild with counts
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

    /// Updates a fleet category's name and duration fields
    /// Returns None if the category doesn't exist or doesn't belong to the guild
    pub async fn update(
        &self,
        params: UpdateFleetCategoryParams,
    ) -> Result<Option<FleetCategory>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Check if category exists and belongs to the guild
        if !repo.exists_in_guild(params.id, params.guild_id).await? {
            return Ok(None);
        }

        let _result = repo.update(params.clone()).await?;

        // Fetch full category with relations
        let full_result = repo.get_by_id(params.id).await?;

        full_result
            .map(FleetCategory::from_with_relations)
            .transpose()
            .map_err(Into::into)
    }

    /// Deletes a fleet category
    /// Returns true if deleted, false if not found or doesn't belong to guild
    pub async fn delete(&self, id: i32, guild_id: u64) -> Result<bool, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        // Check if category exists and belongs to the guild
        if !repo.exists_in_guild(id, guild_id).await? {
            return Ok(false);
        }

        repo.delete(id).await?;

        Ok(true)
    }

    /// Gets fleet categories by ping format ID
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<FleetCategoryListItem>, AppError> {
        let repo = FleetCategoryRepository::new(self.db);

        let categories = repo.get_by_ping_format_id(ping_format_id).await?;

        let categories: Result<Vec<_>, _> = categories
            .into_iter()
            .map(FleetCategoryListItem::from_entity)
            .collect();

        Ok(categories?)
    }
}
