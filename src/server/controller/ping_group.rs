use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tower_sessions::Session;

use crate::{
    model::{
        api::ErrorDto,
        pagination::PageDto,
        ping_group::{
            CreatePingGroupDto, PaginatedPingGroupsDto, PingGroupDto, UpdatePingGroupDto,
        },
    },
    server::{
        controller::param::PaginationParam,
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        model::ping_group::{CreatePingGroupParam, UpdatePingGroupParam},
        service::ping_group::PingGroupService,
        state::AppState,
    },
};

pub static PING_GROUP_TAG: &str = "ping_group";

#[utoipa::path(
    post,
    path = "/api/admin/servers/{guild_id}/ping-group",
    tag = PING_GROUP_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    request_body = CreatePingGroupDto,
    responses(
        (status = 201, description = "Successfully created ping format", body = PingGroupDto),
        (status = 400, description = "Invalid ping group data", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn create_ping_group(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Json(payload): Json<CreatePingGroupDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let ping_group = PingGroupService::new(&state.db)
        .create(guild_id, CreatePingGroupParam::from(payload))
        .await?;

    Ok((StatusCode::CREATED, Json(ping_group.into_dto())))
}

#[utoipa::path(
    get,
    path = "/api/admin/servers/{guild_id}/ping-groups",
    tag = PING_GROUP_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("entries" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved ping groups", body = PaginatedPingGroupsDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_paginated_ping_groups(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Query(pagination): Query<PaginationParam>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let page = PingGroupService::new(&state.db)
        .list_by_guild(guild_id, pagination.page, pagination.entries)
        .await?;

    let dto = page.map(|ping_group| ping_group.into_dto());

    Ok((StatusCode::OK, Json(PageDto::from(dto))))
}

#[utoipa::path(
    put,
    path = "/api/admin/servers/{guild_id}/ping-group/{id}",
    tag = PING_GROUP_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("id" = i32, Path, description = "Ping group ID"),
    ),
    request_body = UpdatePingGroupDto,
    responses(
        (status = 200, description = "Successfully updated ping group", body = PingGroupDto),
        (status = 400, description = "Invalid ping group data", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "Ping group not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn update_ping_group(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, id)): Path<(u64, i32)>,
    Json(payload): Json<UpdatePingGroupDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let ping_group = PingGroupService::new(&state.db)
        .update(guild_id, id, UpdatePingGroupParam::from(payload))
        .await?;

    Ok((StatusCode::OK, Json(ping_group.into_dto())))
}

#[utoipa::path(
    delete,
    path = "/api/admin/servers/{guild_id}/ping-group/{id}",
    tag = PING_GROUP_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("id" = i32, Path, description = "Ping group ID"),
    ),
    responses(
        (status = 204, description = "Successfully deleted ping format"),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn delete_ping_group(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let _ = PingGroupService::new(&state.db)
        .delete(guild_id, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
