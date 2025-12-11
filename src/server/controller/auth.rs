use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

/// Session key for CSRF token
static SESSION_OAUTH_CSRF_TOKEN: &str = "oauth:csrf_token";

use crate::server::{
    data::discord::user::DiscordUserRepository,
    error::{auth::AuthError, AppError},
    service::oauth::DiscordAuthService,
    state::AppState,
};

/// Query parameters for the OAuth callback endpoint.
///
/// # Fields
/// - `state` - CSRF protection token that must match the value stored in the session
/// - `code` - Authorization code used to exchange for access tokens
#[derive(Deserialize)]
pub struct CallbackParams {
    /// CSRF state token to be validated against the session value.
    pub state: String,
    /// Authorization code from Discord SSO for token exchange.
    pub code: String,
}

pub async fn login(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = DiscordAuthService::new(state.http_client, state.oauth_client);

    let (url, csrf_token) = auth_service.login_url();

    // Store CSRF token in session for verification during callback
    session
        .insert(SESSION_OAUTH_CSRF_TOKEN, csrf_token.secret())
        .await?;

    Ok(Redirect::temporary(&url.to_string()))
}

pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    params: Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = DiscordAuthService::new(state.http_client, state.oauth_client);
    let discord_user_repo = DiscordUserRepository::new(&state.db);

    validate_csrf(&session, &params.0.state).await?;

    let user = auth_service.callback(params.0.code).await?;
    let _new_user = discord_user_repo.upsert(user.clone()).await?;

    Ok((StatusCode::OK, Json(user)))
}

async fn validate_csrf(session: &Session, csrf_state: &str) -> Result<(), AppError> {
    let stored_state: Option<String> = session.remove(SESSION_OAUTH_CSRF_TOKEN).await?;

    if let Some(state) = stored_state {
        if state == csrf_state {
            return Ok(());
        }
    }

    Err(AppError::AuthErr(AuthError::CsrfValidationFailed))
}
