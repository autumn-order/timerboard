use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::api::SuccessDto,
    server::{
        controller::auth::{SESSION_AUTH_ADDING_BOT, SESSION_AUTH_CSRF_TOKEN},
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        service::{admin::bot::DiscordBotService, user::UserService},
        state::AppState,
    },
};

pub async fn add_bot(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let bot_service = DiscordBotService::new(&state.oauth_client);

    let _ = auth_guard.require(&[Permission::Admin]).await?;

    let (url, csrf_token) = bot_service.bot_url().await?;

    session
        .insert(SESSION_AUTH_CSRF_TOKEN, csrf_token.secret())
        .await?;

    // Set flag to indicate this is a bot addition flow
    session.insert(SESSION_AUTH_ADDING_BOT, true).await?;

    Ok(Redirect::temporary(url.as_str()))
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    page: u64,
    #[serde(default = "default_per_page")]
    per_page: u64,
}

fn default_page() -> u64 {
    0
}

fn default_per_page() -> u64 {
    10
}

pub async fn get_all_users(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let user_service = UserService::new(&state.db);

    let _ = auth_guard.require(&[Permission::Admin]).await?;

    let users = user_service
        .get_all_users(query.page, query.per_page)
        .await?;

    Ok(Json(users))
}

pub async fn get_all_admins(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let user_service = UserService::new(&state.db);

    let _ = auth_guard.require(&[Permission::Admin]).await?;

    let admins = user_service.get_all_admins().await?;

    Ok(Json(admins))
}

pub async fn add_admin(
    State(state): State<AppState>,
    session: Session,
    Path(user_id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let user_service = UserService::new(&state.db);

    let _ = auth_guard.require(&[Permission::Admin]).await?;

    user_service.add_admin(user_id).await?;

    Ok(Json(SuccessDto { success: true }))
}

pub async fn remove_admin(
    State(state): State<AppState>,
    session: Session,
    Path(user_id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let user_service = UserService::new(&state.db);

    let requester = auth_guard.require(&[Permission::Admin]).await?;

    let requester_id = requester.discord_id.parse::<u64>().map_err(|e| {
        AppError::InternalError(format!("Failed to parse requester discord_id: {}", e))
    })?;

    // Prevent self-deletion
    if user_id == requester_id {
        return Err(AppError::BadRequest(
            "You cannot remove your own admin privileges".to_string(),
        ));
    }

    user_service.remove_admin(user_id).await?;

    Ok(Json(SuccessDto { success: true }))
}
