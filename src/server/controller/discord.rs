use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use dioxus_logger::tracing;
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::{
        api::ErrorDto,
        discord::{DiscordGuildChannelDto, DiscordGuildDto, DiscordGuildRoleDto},
    },
    server::{
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        service::discord::{
            DiscordGuildChannelService, DiscordGuildRoleService, DiscordGuildService,
        },
        state::AppState,
    },
};

/// Tag for grouping discord endpoints in OpenAPI documentation
pub static DISCORD_TAG: &str = "discord";

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

/// Get all Discord guilds.
///
/// Returns a list of all Discord guilds (servers) that the bot is a member of.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view all Discord guilds
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
///
/// # Returns
/// - `200 OK` - List of all Discord guilds
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers",
    tag = DISCORD_TAG,
    responses(
        (status = 200, description = "Successfully retrieved Discord guilds", body = Vec<DiscordGuildDto>),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_all_discord_guilds(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let guild_service = DiscordGuildService::new(&state.db);

    let guilds = guild_service.get_all().await?;

    Ok((StatusCode::OK, Json(guilds)))
}

/// Get Discord guild by ID.
///
/// Returns information about a specific Discord guild by its guild ID.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view Discord guild details
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch
///
/// # Returns
/// - `200 OK` - Discord guild information
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - Guild with specified ID not found
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}",
    tag = DISCORD_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved Discord guild", body = DiscordGuildDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Guild not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_discord_guild_by_id(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    tracing::info!("Guild ID: {}", guild_id);

    let guild_service = DiscordGuildService::new(&state.db);

    let Some(guild) = guild_service.get_by_guild_id(guild_id).await? else {
        return Err(AppError::NotFound(format!(
            "Guild with ID {} not found",
            guild_id
        )));
    };

    Ok((StatusCode::OK, Json(guild)))
}

/// Get paginated roles for a Discord guild.
///
/// Returns a paginated list of roles available in the specified Discord guild.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view Discord guild roles
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch roles for
/// - `params` - Pagination parameters (page and entries)
///
/// # Returns
/// - `200 OK` - Paginated list of Discord roles
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/roles",
    tag = DISCORD_TAG,
    params(
        ("guild_id" = i64, Path, description = "Discord guild ID"),
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("entries" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved Discord guild roles", body = Vec<DiscordGuildRoleDto>),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_discord_guild_roles(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = DiscordGuildRoleService::new(&state.db);

    let roles = service
        .get_paginated(guild_id as u64, params.page, params.entries)
        .await?;

    Ok((StatusCode::OK, Json(roles)))
}

/// Get paginated channels for a Discord guild.
///
/// Returns a paginated list of channels available in the specified Discord guild.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view Discord guild channels
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch channels for
/// - `params` - Pagination parameters (page and entries)
///
/// # Returns
/// - `200 OK` - Paginated list of Discord channels
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/channels",
    tag = DISCORD_TAG,
    params(
        ("guild_id" = i64, Path, description = "Discord guild ID"),
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("entries" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved Discord guild channels", body = Vec<DiscordGuildChannelDto>),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_discord_guild_channels(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = DiscordGuildChannelService::new(&state.db);

    let channels = service
        .get_paginated(guild_id as u64, params.page, params.entries)
        .await?;

    Ok((StatusCode::OK, Json(channels)))
}
