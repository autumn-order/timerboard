use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use tower_sessions::Session;

use crate::server::{
    controller::auth::SESSION_AUTH_CSRF_TOKEN,
    error::AppError,
    middleware::auth::{AuthGuard, Permission},
    service::admin::bot::DiscordBotService,
    state::AppState,
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

    Ok(Redirect::temporary(url.as_str()))
}
