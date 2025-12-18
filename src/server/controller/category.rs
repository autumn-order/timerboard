use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::{
        api::ErrorDto,
        category::{
            CreateFleetCategoryDto, FleetCategoryDto, PaginatedFleetCategoriesDto,
            UpdateFleetCategoryDto,
        },
    },
    server::{
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        model::category::{CreateFleetCategoryParams, UpdateFleetCategoryParams},
        service::category::FleetCategoryService,
        state::AppState,
    },
};

/// Tag for grouping category endpoints in OpenAPI documentation
pub static CATEGORY_TAG: &str = "category";

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_entries")]
    pub entries: u64,
}

fn default_entries() -> u64 {
    10
}

/// Create a new fleet category.
///
/// Creates a new fleet category for the specified Discord guild with the provided
/// configuration including name, ping format, cooldowns, and role permissions.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can create fleet categories
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to create the category for
/// - `payload` - Category creation data (name, ping format, cooldowns, roles, etc.)
///
/// # Returns
/// - `201 Created` - Successfully created fleet category
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `400 Bad Request` - Invalid category data
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    post,
    path = "/api/admin/servers/{guild_id}/categories",
    tag = CATEGORY_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    request_body = CreateFleetCategoryDto,
    responses(
        (status = 201, description = "Successfully created fleet category", body = FleetCategoryDto),
        (status = 400, description = "Invalid category data", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn create_fleet_category(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Json(payload): Json<CreateFleetCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = FleetCategoryService::new(&state.db);

    // Convert DTO to server model
    let params = CreateFleetCategoryParams::from_dto(guild_id, payload);

    let category = service.create(params).await?;

    Ok((StatusCode::CREATED, Json(category.into_dto())))
}

/// Get paginated fleet categories for a guild.
///
/// Returns a paginated list of all fleet categories configured for the specified
/// Discord guild. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view fleet categories
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch categories for
/// - `params` - Pagination parameters (page and entries)
///
/// # Returns
/// - `200 OK` - Paginated list of fleet categories
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/categories",
    tag = CATEGORY_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("entries" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved fleet categories", body = PaginatedFleetCategoriesDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_fleet_categories(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = FleetCategoryService::new(&state.db);

    let categories = service
        .get_paginated(guild_id, params.page, params.entries)
        .await?;

    Ok((StatusCode::OK, Json(categories.into_dto())))
}

/// Get a specific fleet category by ID.
///
/// Returns detailed information about a specific fleet category including its
/// configuration, ping format, and role permissions. Verifies the category belongs
/// to the specified guild. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view fleet category details
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID the category should belong to
/// - `category_id` - Fleet category ID to fetch
///
/// # Returns
/// - `200 OK` - Fleet category details
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - Category not found or doesn't belong to the specified guild
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/categories/{category_id}",
    tag = CATEGORY_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("category_id" = i32, Path, description = "Fleet category ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved fleet category", body = FleetCategoryDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Category not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_fleet_category_by_id(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, category_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = FleetCategoryService::new(&state.db);

    let category = service.get_by_id(category_id).await?;

    match category {
        Some(cat) => {
            // Verify it belongs to the guild
            if cat.guild_id == guild_id {
                Ok((StatusCode::OK, Json(cat.into_dto())))
            } else {
                Err(AppError::NotFound("Category not found".to_string()))
            }
        }
        None => Err(AppError::NotFound("Category not found".to_string())),
    }
}

/// Update a fleet category.
///
/// Updates an existing fleet category with new configuration including name,
/// ping format, cooldowns, and role permissions. Verifies the category belongs
/// to the specified guild. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can update fleet categories
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID the category should belong to
/// - `category_id` - Fleet category ID to update
/// - `payload` - Updated category data
///
/// # Returns
/// - `200 OK` - Successfully updated fleet category
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - Category not found or doesn't belong to the specified guild
/// - `400 Bad Request` - Invalid category data
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    put,
    path = "/api/admin/servers/{guild_id}/categories/{category_id}",
    tag = CATEGORY_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("category_id" = i32, Path, description = "Fleet category ID")
    ),
    request_body = UpdateFleetCategoryDto,
    responses(
        (status = 200, description = "Successfully updated fleet category", body = FleetCategoryDto),
        (status = 400, description = "Invalid category data", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Category not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn update_fleet_category(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, category_id)): Path<(u64, i32)>,
    Json(payload): Json<UpdateFleetCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = FleetCategoryService::new(&state.db);

    // Convert DTO to server model
    let params = UpdateFleetCategoryParams::from_dto(category_id, guild_id, payload);

    let category = service.update(params).await?;

    match category {
        Some(cat) => Ok((StatusCode::OK, Json(cat.into_dto()))),
        None => Err(AppError::NotFound("Category not found".to_string())),
    }
}

/// Get fleet categories by ping format ID.
///
/// Returns all fleet categories that use the specified ping format.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view fleet categories by ping format
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID (for route consistency, not used in query)
/// - `ping_format_id` - Ping format ID to filter categories by
///
/// # Returns
/// - `200 OK` - List of fleet categories using the specified ping format
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/formats/{format_id}/categories",
    tag = CATEGORY_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("format_id" = i32, Path, description = "Ping format ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved categories by ping format", body = Vec<FleetCategoryDto>),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_fleet_categories_by_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path((_guild_id, ping_format_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = FleetCategoryService::new(&state.db);

    let categories = service.get_by_ping_format_id(ping_format_id).await?;

    Ok((
        StatusCode::OK,
        Json(
            categories
                .into_iter()
                .map(|c| c.into_dto())
                .collect::<Vec<_>>(),
        ),
    ))
}

/// Delete a fleet category.
///
/// Deletes an existing fleet category from the specified guild. Verifies the
/// category belongs to the specified guild before deletion. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can delete fleet categories
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID the category should belong to
/// - `category_id` - Fleet category ID to delete
///
/// # Returns
/// - `204 No Content` - Successfully deleted fleet category
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - Category not found or doesn't belong to the specified guild
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    delete,
    path = "/api/admin/servers/{guild_id}/categories/{category_id}",
    tag = CATEGORY_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("category_id" = i32, Path, description = "Fleet category ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted fleet category"),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Category not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn delete_fleet_category(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, category_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = FleetCategoryService::new(&state.db);

    let deleted = service.delete(category_id, guild_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
