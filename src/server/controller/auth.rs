use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

/// Session key for CSRF token
pub static SESSION_AUTH_CSRF_TOKEN: &str = "auth:csrf_token";
/// Session key for whether to set user as admin
static SESSION_AUTH_SET_ADMIN: &str = "auth:set_admin";
/// Session key for current user ID
pub static SESSION_AUTH_USER_ID: &str = "auth:user";
/// Session key for bot addition flow
pub static SESSION_AUTH_ADDING_BOT: &str = "auth:adding_bot";

use crate::server::{
    error::{auth::AuthError, AppError},
    service::{auth::AuthService, user::UserService},
    state::AppState,
};

/// Query parameters for the login endpoint.
///
/// # Fields
/// - `admin_code` - Code to set the user as admin on login
#[derive(Deserialize)]
pub struct LoginParams {
    /// Code will be validated, setting the user as admin if successful
    pub admin_code: Option<String>,
}

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
    params: Query<LoginParams>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = AuthService::new(&state.db, &state.http_client, &state.oauth_client);
    let admin_code = &params.0.admin_code;

    // Validate admin code if provided
    if let Some(code) = admin_code {
        let is_valid = state.admin_code_service.validate_and_consume(code).await;

        if !is_valid {
            return Err(AppError::AuthErr(AuthError::AdminCodeValidationFailed));
        }

        // Store admin code validation success in session
        session.insert(SESSION_AUTH_SET_ADMIN, true).await?;
    }

    let (url, csrf_token) = auth_service.login_url();

    // Store CSRF token in session for verification during callback
    session
        .insert(SESSION_AUTH_CSRF_TOKEN, csrf_token.secret())
        .await?;

    Ok(Redirect::temporary(url.as_str()))
}

pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    params: Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&session, &params.0.state).await?;

    // Check if this is a bot addition callback
    let adding_bot: bool = session
        .remove(SESSION_AUTH_ADDING_BOT)
        .await?
        .unwrap_or(false);

    if adding_bot {
        // Bot addition doesn't return a code for OAuth, just redirect to admin
        return Ok(Redirect::permanent("/admin"));
    }

    let auth_service = AuthService::new(&state.db, &state.http_client, &state.oauth_client);

    // Check if admin code was validated in the login flow
    let set_admin: bool = session
        .remove(SESSION_AUTH_SET_ADMIN)
        .await?
        .unwrap_or(false);

    let new_user = auth_service.callback(params.0.code, set_admin).await?;

    session
        .insert(SESSION_AUTH_USER_ID, new_user.discord_id.clone())
        .await?;

    Ok(Redirect::permanent("/"))
}

pub async fn logout(session: Session) -> Result<impl IntoResponse, AppError> {
    // Only clear session if there actually is a user in session
    if let Some(_user_id) = session.get::<String>(SESSION_AUTH_USER_ID).await? {
        session.clear().await;
    }

    Ok(Redirect::temporary("/login"))
}

/// Retrieve information for user with active session
pub async fn get_user(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let user_service = UserService::new(&state.db);

    let Some(user_id_str) = session.get::<String>(SESSION_AUTH_USER_ID).await? else {
        return Err(AuthError::UserNotInSession.into());
    };

    let user_id = user_id_str.parse::<u64>().map_err(|e| {
        AppError::InternalError(format!("Failed to parse user_id from session: {}", e))
    })?;

    let Some(user) = user_service.get_user(user_id).await? else {
        return Err(AuthError::UserNotInDatabase(user_id).into());
    };

    Ok((StatusCode::OK, Json(user)))
}

async fn validate_csrf(session: &Session, csrf_state: &str) -> Result<(), AppError> {
    let stored_state: Option<String> = session.remove(SESSION_AUTH_CSRF_TOKEN).await?;

    if let Some(state) = stored_state {
        if state == csrf_state {
            return Ok(());
        }
    }

    Err(AppError::AuthErr(AuthError::CsrfValidationFailed))
}
