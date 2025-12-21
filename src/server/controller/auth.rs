use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::model::{api::ErrorDto, user::UserDto};

/// Tag for grouping auth endpoints in OpenAPI documentation
pub static AUTH_TAG: &str = "auth";

use crate::server::{
    error::{auth::AuthError, AppError},
    middleware::session::{AuthSession, CsrfSession, OAuthFlowSession},
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

/// Initiates Discord OAuth2 login flow.
///
/// Generates a Discord OAuth2 login URL with CSRF protection and redirects the user
/// to Discord for authentication. Optionally validates an admin code to grant admin
/// privileges upon successful login.
///
/// # Access Control
/// - Public endpoint - no authentication required
///
/// # Arguments
/// - `state` - Application state containing OAuth client and admin code service
/// - `session` - User's session for storing CSRF token and admin code validation status
/// - `params` - Query parameters containing optional admin code
///
/// # Returns
/// - `307 Temporary Redirect` - Redirects to Discord OAuth login page
/// - `Err(AuthError::AdminCodeValidationFailed)` - Invalid admin code provided
/// - `Err(SessionErr(_))` - Failed to store data in session
#[utoipa::path(
    get,
    path = "/api/auth/login",
    tag = AUTH_TAG,
    params(
        ("admin_code" = Option<String>, Query, description = "Optional admin code for granting admin privileges")
    ),
    responses(
        (status = 307, description = "Redirect to Discord OAuth login page"),
        (status = 400, description = "Invalid admin code", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    params: Query<LoginParams>,
) -> Result<impl IntoResponse, AppError> {
    let oauth_session = OAuthFlowSession::new(&session);
    let csrf_session = CsrfSession::new(&session);
    let auth_service = AuthService::new(&state.db, &state.http_client, &state.oauth_client);
    let admin_code = &params.0.admin_code;

    // Validate admin code if provided
    if let Some(code) = admin_code {
        let is_valid = state.admin_code_service.validate_and_consume(code).await;

        if !is_valid {
            return Err(AppError::AuthErr(AuthError::AdminCodeValidationFailed));
        }

        // Store admin code validation success in session
        oauth_session.set_admin_flag(true).await?;
    }

    let (url, csrf_token) = auth_service.login_url();

    // Store CSRF token in session for verification during callback
    csrf_session.set_token(csrf_token.secret().clone()).await?;

    Ok(Redirect::temporary(url.as_str()))
}

/// Handles OAuth2 callback from Discord.
///
/// Validates the CSRF state token, exchanges the authorization code for access tokens,
/// fetches user information from Discord, creates or updates the user in the database,
/// and establishes a session. If an admin code was validated during login, grants admin
/// privileges to the user.
///
/// # Access Control
/// - Public endpoint - no authentication required (but requires valid OAuth flow)
///
/// # Arguments
/// - `state` - Application state containing database, HTTP client, and OAuth client
/// - `session` - User's session for CSRF validation and storing user ID
/// - `params` - Query parameters containing OAuth state and authorization code
///
/// # Returns
/// - `308 Permanent Redirect` - Redirects to homepage on success, or admin page if bot addition
/// - `Err(AuthError::CsrfValidationFailed)` - CSRF token validation failed
/// - `Err(AuthError::_)` - Various authentication errors
/// - `Err(DbErr(_))` - Database operation failed
/// - `Err(SessionErr(_))` - Session operation failed
#[utoipa::path(
    get,
    path = "/api/auth/callback",
    tag = AUTH_TAG,
    params(
        ("state" = String, Query, description = "CSRF state token for validation"),
        ("code" = String, Query, description = "OAuth authorization code from Discord")
    ),
    responses(
        (status = 308, description = "Redirect to homepage after successful login"),
        (status = 400, description = "Invalid CSRF token or OAuth code", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    params: Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    let csrf_session = CsrfSession::new(&session);
    let oauth_session = OAuthFlowSession::new(&session);
    let auth_session = AuthSession::new(&session);

    validate_csrf(&csrf_session, &params.0.state).await?;

    // Check if this is a bot addition callback
    let adding_bot = oauth_session.take_adding_bot_flag().await?;

    if adding_bot {
        // Bot addition doesn't return a code for OAuth, just redirect to admin
        return Ok(Redirect::permanent("/admin"));
    }

    let auth_service = AuthService::new(&state.db, &state.http_client, &state.oauth_client);

    // Check if admin code was validated in the login flow
    let set_admin = oauth_session.take_admin_flag().await?;

    let new_user = auth_service.callback(params.0.code, set_admin).await?;

    auth_session.set_user_id(new_user.discord_id).await?;

    Ok(Redirect::permanent("/"))
}

/// Logs out the current user.
///
/// Clears the user's session if they are logged in and redirects to the login page.
///
/// # Access Control
/// - Public endpoint - no authentication required
///
/// # Arguments
/// - `session` - User's session to be cleared
///
/// # Returns
/// - `307 Temporary Redirect` - Redirects to login page
/// - `Err(SessionErr(_))` - Failed to clear session
#[utoipa::path(
    get,
    path = "/api/auth/logout",
    tag = AUTH_TAG,
    responses(
        (status = 307, description = "Redirect to login page after logout"),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn logout(session: Session) -> Result<impl IntoResponse, AppError> {
    let auth_session = AuthSession::new(&session);

    // Only clear session if there actually is a user in session
    if auth_session.is_authenticated().await? {
        auth_session.clear().await;
    }

    Ok(Redirect::temporary("/login"))
}

/// Retrieves information for the currently authenticated user.
///
/// Fetches the user ID from the session and queries the database to retrieve
/// the user's Discord ID, name, and admin status.
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session containing their Discord ID
///
/// # Returns
/// - `200 OK` - User information (Discord ID, name, admin status)
/// - `404 Not Found` - User not in session or not found in database
/// - `500 Internal Server Error` - Database or session error
#[utoipa::path(
    get,
    path = "/api/auth/user",
    tag = AUTH_TAG,
    responses(
        (status = 200, description = "Successfully retrieved user information", body = UserDto),
        (status = 404, description = "User not found in session or database", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_user(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let auth_session = AuthSession::new(&session);
    let user_service = UserService::new(&state.db);

    let Some(user_id) = auth_session.get_user_id().await? else {
        return Err(AuthError::UserNotInSession.into());
    };

    let param = crate::server::model::user::GetUserParam {
        discord_id: user_id,
    };

    let Some(user) = user_service.get_user(param).await? else {
        return Err(AuthError::UserNotInDatabase(user_id).into());
    };

    Ok((StatusCode::OK, Json(user.into_dto())))
}

async fn validate_csrf<'a>(
    csrf_session: &CsrfSession<'a>,
    csrf_state: &str,
) -> Result<(), AppError> {
    let stored_state = csrf_session.take_token().await?;

    if let Some(state) = stored_state {
        if state == csrf_state {
            return Ok(());
        }
    }

    Err(AppError::AuthErr(AuthError::CsrfValidationFailed))
}
