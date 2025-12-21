use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tower_sessions::Session;

use crate::{
    model::{api::ErrorDto, category::FleetCategoryListItemDto, discord::DiscordGuildDto},
    server::{
        error::AppError,
        middleware::auth::AuthGuard,
        model::user::GetUserParam,
        service::{category::FleetCategoryService, user::UserService},
        state::AppState,
    },
};

/// Tag for grouping user endpoints in OpenAPI documentation
pub static USER_TAG: &str = "user";

/// Get all guilds available to the current user.
///
/// Returns a list of all Discord guilds (timerboards) that the authenticated user
/// has access to. If the user is an admin, returns ALL guilds in the system.
/// If the user is not an admin, returns only guilds the user is a member of.
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
///
/// # Returns
/// - `200 OK` - JSON array of DiscordGuildDto (all guilds for admins, user's guilds otherwise)
/// - `401 Unauthorized` - User not authenticated
/// - `500 Internal Server Error` - Database or parsing error
#[utoipa::path(
    get,
    path = "/api/user/guilds",
    tag = USER_TAG,
    responses(
        (status = 200, description = "Successfully retrieved user guilds", body = Vec<DiscordGuildDto>),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_user_guilds(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let user = auth_guard.require(&[]).await?;

    let param = GetUserParam {
        discord_id: user.discord_id,
    };

    let user_service = UserService::new(&state.db);
    let guilds = user_service.get_user_guilds(param).await?;

    let guild_dtos = guilds.into_iter().map(|g| g.into_dto()).collect::<Vec<_>>();

    Ok((StatusCode::OK, Json(guild_dtos)))
}

/// Get fleet categories user can create/manage.
///
/// Returns a list of fleet categories where the authenticated user has can_create OR
/// can_manage permissions through their Discord roles. Admins get all categories for the guild.
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `guild_id` - Discord guild ID to fetch categories for
/// - `session` - User's session for authentication
///
/// # Returns
/// - `200 OK` - JSON array of FleetCategoryListItem (all categories for admins, manageable categories otherwise)
/// - `401 Unauthorized` - User not authenticated
/// - `500 Internal Server Error` - Database or parsing error
#[utoipa::path(
    get,
    path = "/api/user/guilds/{guild_id}/manageable-categories",
    tag = USER_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved manageable categories", body = Vec<FleetCategoryListItemDto>),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_user_manageable_categories(
    State(state): State<AppState>,
    Path(guild_id): Path<u64>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let auth_guard = AuthGuard::new(&state.db, &session);
    let user = auth_guard.require(&[]).await?;

    let category_service = FleetCategoryService::new(&state.db);
    let categories = category_service
        .get_manageable_by_user(user.discord_id, guild_id, user.admin)
        .await?;

    let categories_dto: Vec<_> = categories.into_iter().map(|c| c.into_dto()).collect();

    Ok((StatusCode::OK, Json(categories_dto)))
}
