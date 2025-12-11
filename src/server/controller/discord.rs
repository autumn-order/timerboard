use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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
