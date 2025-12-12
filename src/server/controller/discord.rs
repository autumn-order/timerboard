use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use dioxus_logger::tracing;
use tower_sessions::Session;

use crate::server::{
    error::AppError,
    middleware::auth::{AuthGuard, Permission},
    service::discord::DiscordGuildService,
    state::AppState,
};

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
