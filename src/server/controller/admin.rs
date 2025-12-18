use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::{
        api::{ErrorDto, SuccessDto},
        user::{PaginatedUsersDto, UserDto},
    },
    server::{
        controller::auth::{SESSION_AUTH_ADDING_BOT, SESSION_AUTH_CSRF_TOKEN},
        error::AppError,
        middleware::auth::{AuthGuard, Permission},
        service::{admin::bot::DiscordBotService, user::UserService},
        state::AppState,
    },
};

/// Tag for grouping admin endpoints in OpenAPI documentation
pub static ADMIN_TAG: &str = "admin";

/// Add Discord bot to a server.
///
/// Generates a Discord bot invitation URL with required permissions and redirects
/// the admin to Discord to add the bot to a server. Stores CSRF token and bot
/// addition flag in the session for callback validation.
///
/// # Access Control
/// - `Admin` - Only admins can add the bot to servers
///
/// # Arguments
/// - `state` - Application state containing OAuth client
/// - `session` - User's session for storing CSRF token and bot addition flag
///
/// # Returns
/// - `307 Temporary Redirect` - Redirects to Discord bot invitation page
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Failed to generate bot URL or store session data
#[utoipa::path(
    get,
    path = "/api/admin/bot/add",
    tag = ADMIN_TAG,
    responses(
        (status = 307, description = "Redirect to Discord bot invitation page"),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
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

/// Get all users with pagination.
///
/// Returns a paginated list of all users in the system with their Discord ID,
/// name, and admin status. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view all users
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `query` - Pagination parameters (page and per_page)
///
/// # Returns
/// - `200 OK` - Paginated list of users with total count and pagination metadata
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = ADMIN_TAG,
    params(
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("per_page" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved paginated users", body = PaginatedUsersDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
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

/// Get all admin users.
///
/// Returns a list of all users who have admin privileges in the system.
/// Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can view the admin list
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
///
/// # Returns
/// - `200 OK` - List of admin users
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/admin/admins",
    tag = ADMIN_TAG,
    responses(
        (status = 200, description = "Successfully retrieved admin users", body = Vec<UserDto>),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
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

/// Grant admin privileges to a user.
///
/// Adds admin privileges to the specified user by their Discord ID. Only accessible
/// by existing admins.
///
/// # Access Control
/// - `Admin` - Only admins can grant admin privileges
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `user_id` - Discord ID of the user to make admin
///
/// # Returns
/// - `200 OK` - Admin privileges successfully granted
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - User with specified ID not found
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    post,
    path = "/api/admin/admins/{user_id}",
    tag = ADMIN_TAG,
    params(
        ("user_id" = u64, Path, description = "Discord ID of user to grant admin privileges")
    ),
    responses(
        (status = 200, description = "Successfully granted admin privileges", body = SuccessDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "User not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
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

/// Revoke admin privileges from a user.
///
/// Removes admin privileges from the specified user by their Discord ID. Users
/// cannot remove their own admin privileges. Only accessible by admins.
///
/// # Access Control
/// - `Admin` - Only admins can revoke admin privileges
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `user_id` - Discord ID of the user to revoke admin privileges from
///
/// # Returns
/// - `200 OK` - Admin privileges successfully revoked
/// - `400 Bad Request` - User attempted to remove their own admin privileges
/// - `401 Unauthorized` - User not authenticated or not an admin
/// - `404 Not Found` - User with specified ID not found
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    delete,
    path = "/api/admin/admins/{user_id}",
    tag = ADMIN_TAG,
    params(
        ("user_id" = u64, Path, description = "Discord ID of user to revoke admin privileges")
    ),
    responses(
        (status = 200, description = "Successfully revoked admin privileges", body = SuccessDto),
        (status = 400, description = "Cannot remove own admin privileges", body = ErrorDto),
        (status = 401, description = "User not authenticated or not an admin", body = ErrorDto),
        (status = 404, description = "User not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
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
