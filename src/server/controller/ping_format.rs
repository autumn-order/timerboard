use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::ping_format::{CreatePingFormatDto, PingFormatDto, UpdatePingFormatDto},
    server::{
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        service::ping_format::PingFormatService,
        state::AppState,
    },
};

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

/// POST /api/timerboard/{guild_id}/ping/format
/// Create a new ping format
pub async fn create_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<i64>,
    Json(payload): Json<CreatePingFormatDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let field_names: Vec<String> = payload.fields.into_iter().map(|f| f.name).collect();

    let ping_format = service.create(guild_id, payload.name, field_names).await?;

    Ok((StatusCode::CREATED, Json(ping_format)))
}

/// GET /api/timerboard/{guild_id}/ping/format
/// Get paginated ping formats for a guild (with all fields per format)
pub async fn get_ping_formats(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let ping_formats = service
        .get_paginated(guild_id, params.page, params.entries)
        .await?;

    Ok((StatusCode::OK, Json(ping_formats)))
}

/// PUT /api/timerboard/{guild_id}/ping/format/{format_id}
/// Update a ping format's name and fields
pub async fn update_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, format_id)): Path<(i64, i32)>,
    Json(payload): Json<UpdatePingFormatDto>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let fields: Vec<(Option<i32>, String)> =
        payload.fields.into_iter().map(|f| (f.id, f.name)).collect();

    let ping_format = service
        .update(format_id, guild_id, payload.name, fields)
        .await?;

    match ping_format {
        Some(pf) => Ok((StatusCode::OK, Json(pf))),
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(PingFormatDto {
                id: 0,
                guild_id: 0,
                name: String::new(),
                fields: Vec::new(),
            }),
        )),
    }
}

/// DELETE /api/timerboard/{guild_id}/ping/format/{format_id}
/// Delete a ping format
pub async fn delete_ping_format(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, format_id)): Path<(i64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = AuthGuard::new(&state.db, &session)
        .require(&[Permission::Admin])
        .await?;

    let service = PingFormatService::new(&state.db);

    let deleted = service.delete(format_id, guild_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
