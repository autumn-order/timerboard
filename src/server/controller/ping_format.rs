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
        ping_format::{CreatePingFormatDto, PingFormatDto, UpdatePingFormatDto},
    },
    server::{
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        model::ping_format::{
            CreatePingFormatWithFieldsParam, DeletePingFormatParam, GetPaginatedPingFormatsParam,
            UpdatePingFormatWithFieldsParam,
        },
        service::ping_format::PingFormatService,
        state::AppState,
    },
};

/// Tag for grouping ping format endpoints in OpenAPI documentation
pub static PING_FORMAT_TAG: &str = "ping_format";

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

/// Create a new ping format.
///
/// Creates a new ping format for the specified Discord guild with a name and
/// custom fields. Each field has a name, priority, and optional default value.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can create ping formats
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to create the ping format for
/// - `payload` - Ping format creation data (name and fields)
///
/// # Returns
/// - `201 Created` - Successfully created ping format
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `400 Bad Request` - Invalid ping format data
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    post,
    path = "/api/admin/servers/{guild_id}/formats",
    tag = PING_FORMAT_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    request_body = CreatePingFormatDto,
    responses(
        (status = 201, description = "Successfully created ping format", body = PingFormatDto),
        (status = 400, description = "Invalid ping format data", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn create_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Json(payload): Json<CreatePingFormatDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let fields: Vec<(String, i32, Option<String>)> = payload
        .fields
        .into_iter()
        .map(|f| (f.name, f.priority, f.default_value))
        .collect();

    let param = CreatePingFormatWithFieldsParam {
        guild_id,
        name: payload.name,
        fields,
    };

    let ping_format = service.create(param).await?;
    let dto = ping_format.into_dto();

    Ok((StatusCode::CREATED, Json(dto)))
}

/// Get paginated ping formats for a guild.
///
/// Returns a paginated list of all ping formats configured for the specified
/// Discord guild, including all fields for each format. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view ping formats
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch ping formats for
/// - `params` - Pagination parameters (page and entries)
///
/// # Returns
/// - `200 OK` - Paginated list of ping formats with their fields
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/formats",
    tag = PING_FORMAT_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("entries" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved ping formats", body = Vec<PingFormatDto>),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_ping_formats(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let param = GetPaginatedPingFormatsParam {
        guild_id,
        page: params.page,
        per_page: params.entries,
    };

    let ping_formats = service.get_paginated(param).await?;
    let dto = ping_formats.into_dto();

    Ok((StatusCode::OK, Json(dto)))
}

/// Update a ping format's name and fields.
///
/// Updates an existing ping format with a new name and/or fields. Fields can be
/// added, updated, or removed. Verifies the ping format belongs to the specified
/// guild. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can update ping formats
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID the ping format should belong to
/// - `format_id` - Ping format ID to update
/// - `payload` - Updated ping format data (name and fields)
///
/// # Returns
/// - `200 OK` - Successfully updated ping format
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - Ping format not found or doesn't belong to the specified guild
/// - `400 Bad Request` - Invalid ping format data
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    put,
    path = "/api/admin/servers/{guild_id}/formats/{format_id}",
    tag = PING_FORMAT_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("format_id" = i32, Path, description = "Ping format ID")
    ),
    request_body = UpdatePingFormatDto,
    responses(
        (status = 200, description = "Successfully updated ping format", body = PingFormatDto),
        (status = 400, description = "Invalid ping format data", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Ping format not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn update_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, format_id)): Path<(u64, i32)>,
    Json(payload): Json<UpdatePingFormatDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let fields: Vec<(Option<i32>, String, i32, Option<String>)> = payload
        .fields
        .into_iter()
        .map(|f| (f.id, f.name, f.priority, f.default_value))
        .collect();

    let param = UpdatePingFormatWithFieldsParam {
        id: format_id,
        guild_id,
        name: payload.name,
        fields,
    };

    let ping_format = service.update(param).await?;

    match ping_format {
        Some(pf) => {
            let dto = pf.into_dto();
            Ok((StatusCode::OK, Json(dto)))
        }
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(PingFormatDto {
                id: 0,
                guild_id: 0,
                name: String::new(),
                fields: Vec::new(),
                fleet_category_count: 0,
                fleet_category_names: Vec::new(),
            }),
        )),
    }
}

/// Delete a ping format.
///
/// Deletes an existing ping format from the specified guild. Verifies the ping
/// format belongs to the specified guild before deletion. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can delete ping formats
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID the ping format should belong to
/// - `format_id` - Ping format ID to delete
///
/// # Returns
/// - `204 No Content` - Successfully deleted ping format
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - Ping format not found or doesn't belong to the specified guild
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    delete,
    path = "/api/admin/servers/{guild_id}/formats/{format_id}",
    tag = PING_FORMAT_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("format_id" = i32, Path, description = "Ping format ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted ping format"),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Ping format not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn delete_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, format_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let param = DeletePingFormatParam {
        id: format_id,
        guild_id,
    };

    let deleted = service.delete(param).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
