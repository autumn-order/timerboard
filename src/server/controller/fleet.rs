use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::{
        api::ErrorDto,
        category::FleetCategoryDetailsDto,
        discord::DiscordGuildMemberDto,
        fleet::{CreateFleetDto, FleetDto, PaginatedFleetsDto, UpdateFleetDto},
    },
    server::{
        data::{
            category::FleetCategoryRepository, ping_format::field::PingFormatFieldRepository,
            ping_group::PingGroupRepository,
            user_category_permission::UserCategoryPermissionRepository,
        },
        error::{auth::AuthError, AppError},
        middleware::auth::{AuthGuard, Permission},
        model::fleet::{CreateFleetParam, GetPaginatedFleetsByGuildParam},
        service::fleet::FleetService,
        state::AppState,
    },
};

/// Tag for grouping fleet endpoints in OpenAPI documentation
pub static FLEET_TAG: &str = "fleet";

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_per_page() -> u64 {
    10
}

/// Get category details including ping format fields for fleet creation.
///
/// Returns detailed information about a fleet category including its ping format fields,
/// access roles, ping roles, and channels. Used for fleet creation forms.
///
/// # Access Control
/// - `CategoryView` - User must have view permission for the category
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID
/// - `category_id` - Fleet category ID to fetch details for
///
/// # Returns
/// - `200 OK` - Category details with ping format fields and role configurations
/// - `401 Unauthorized` - User not authenticated or lacks view permission
/// - `404 Not Found` - Category not found
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/guilds/{guild_id}/categories/{category_id}/details",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("category_id" = i32, Path, description = "Fleet category ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved category details", body = FleetCategoryDetailsDto),
        (status = 401, description = "User not authenticated or lacks permission", body = ErrorDto),
        (status = 404, description = "Category not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_category_details(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, category_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let _user = AuthGuard::new(&state.db, &session)
        .require(&[Permission::CategoryView(guild_id, category_id)])
        .await?;

    let category_repo = FleetCategoryRepository::new(&state.db);
    let category_with_relations = category_repo
        .find_by_id(category_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    // Get ping format fields
    let field_repo = PingFormatFieldRepository::new(&state.db);
    let fields = field_repo
        .get_by_ping_format_id(guild_id, category_with_relations.category.ping_format_id)
        .await?
        .into_iter()
        .map(|f| {
            let dto = f.into_dto();
            crate::model::category::PingFormatFieldDto {
                id: dto.id,
                name: dto.name,
                priority: dto.priority,
                field_type: dto.field_type,
                default_field_values: dto.default_field_values,
            }
        })
        .collect();

    // Fetch ping group details if ping_group_id is present
    let (ping_group_name, ping_group_cooldown) = if let Some(ping_group_id) =
        category_with_relations.category.ping_group_id
    {
        let ping_group_repo = PingGroupRepository::new(&state.db);
        if let Ok(Some(ping_group)) = ping_group_repo.find_by_id(guild_id, ping_group_id).await {
            (
                Some(ping_group.name),
                ping_group
                    .cooldown
                    .map(|d| chrono::Duration::seconds(d.num_seconds())),
            )
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Build the response DTO
    let dto = FleetCategoryDetailsDto {
        id: category_with_relations.category.id,
        guild_id,
        ping_format_id: category_with_relations.category.ping_format_id,
        ping_format_name: category_with_relations
            .ping_format
            .map(|pf| pf.name)
            .unwrap_or_default(),
        name: category_with_relations.category.name.clone(),
        ping_group_id: category_with_relations.category.ping_group_id,
        ping_group_name,
        ping_group_cooldown,
        ping_lead_time: category_with_relations
            .category
            .ping_cooldown
            .map(|seconds| chrono::Duration::seconds(seconds as i64)),
        ping_reminder: category_with_relations
            .category
            .ping_reminder
            .map(|seconds| chrono::Duration::seconds(seconds as i64)),
        max_pre_ping: category_with_relations
            .category
            .max_pre_ping
            .map(|seconds| chrono::Duration::seconds(seconds as i64)),
        access_roles: category_with_relations
            .access_roles
            .into_iter()
            .filter_map(|(access_role, role_model)| {
                role_model.map(|role| crate::model::category::FleetCategoryAccessRoleDto {
                    role_id: role.role_id.parse().unwrap_or(0),
                    role_name: role.name,
                    role_color: role.color,
                    position: role.position,
                    can_view: access_role.can_view,
                    can_create: access_role.can_create,
                    can_manage: access_role.can_manage,
                })
            })
            .collect(),
        ping_roles: category_with_relations
            .ping_roles
            .into_iter()
            .filter_map(|(_ping_role, role_model)| {
                role_model.map(|role| crate::model::category::FleetCategoryPingRoleDto {
                    role_id: role.role_id.parse().unwrap_or(0),
                    role_name: role.name,
                    role_color: role.color,
                    position: role.position,
                })
            })
            .collect(),
        channels: category_with_relations
            .channels
            .into_iter()
            .filter_map(|(_cat_channel, channel_model)| {
                channel_model.map(|channel| crate::model::category::FleetCategoryChannelDto {
                    channel_id: channel.channel_id.parse().unwrap_or(0),
                    channel_name: channel.name,
                    position: channel.position,
                })
            })
            .collect(),
        fields,
    };

    Ok((StatusCode::OK, Json(dto)))
}

/// Get all members of a Discord guild for FC selection.
///
/// Returns a list of all members in the specified Discord guild. Used for selecting
/// fleet commanders when creating or updating fleets.
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated
///
/// # Arguments
/// - `state` - Application state containing the database connection
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch members for
///
/// # Returns
/// - `200 OK` - List of guild members with their user ID, username, and display name
/// - `401 Unauthorized` - User not authenticated
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/guilds/{guild_id}/members",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved guild members", body = Vec<DiscordGuildMemberDto>),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_guild_members(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let _user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    use crate::server::data::discord::DiscordGuildMemberRepository;
    let member_repo = DiscordGuildMemberRepository::new(&state.db);
    let members = member_repo.get_members_by_guild(guild_id).await?;

    let member_dtos: Vec<DiscordGuildMemberDto> = members
        .into_iter()
        .map(|member| DiscordGuildMemberDto {
            user_id: member.user_id,
            username: member.username.clone(),
            // Use nickname if available, otherwise fall back to username
            display_name: member.nickname.unwrap_or_else(|| member.username),
            avatar_hash: None,
        })
        .collect();

    Ok((StatusCode::OK, Json(member_dtos)))
}

/// Create a new fleet.
///
/// Creates a new fleet in the specified category with fleet time, commander, description,
/// and custom field values. Sends Discord notifications to configured channels and roles.
///
/// # Access Control
/// - `CategoryCreate` - User must have create permission for the category
///
/// # Arguments
/// - `state` - Application state containing database, Discord HTTP client, and app URL
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to create the fleet in
/// - `dto` - Fleet creation data (category, time, commander, description, fields, etc.)
///
/// # Returns
/// - `201 Created` - Successfully created fleet
/// - `401 Unauthorized` - User not authenticated or lacks create permission
/// - `400 Bad Request` - Invalid fleet data
/// - `500 Internal Server Error` - Database or Discord API error
#[utoipa::path(
    post,
    path = "/api/guilds/{guild_id}/fleets",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID")
    ),
    request_body = CreateFleetDto,
    responses(
        (status = 201, description = "Successfully created fleet", body = FleetDto),
        (status = 400, description = "Invalid fleet data", body = ErrorDto),
        (status = 401, description = "User not authenticated or lacks permission", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn create_fleet(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Json(dto): Json<CreateFleetDto>,
) -> Result<impl IntoResponse, AppError> {
    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());

    let user = AuthGuard::new(&state.db, &session)
        .require(&[Permission::CategoryCreate(guild_id, dto.category_id)])
        .await?;

    let param = CreateFleetParam::from_dto(dto);
    let fleet = fleet_service.create(param, user.admin).await?;

    Ok((StatusCode::CREATED, Json(fleet)))
}

/// Get fleet details by ID.
///
/// Returns detailed information about a specific fleet including its category, time,
/// commander, description, and custom field values. Respects visibility rules for
/// hidden fleets.
///
/// # Visibility Rules
/// Returns a fleet (200 OK) only if ALL of the following are true:
/// 1. User has at least one permission (view, create, or manage) for the fleet's category
/// 2. If the fleet is marked as `hidden`:
///    - User has create OR manage permission for the category, OR
///    - The category's reminder time has elapsed (calculated as fleet_time - ping_reminder), OR
///    - If no reminder is configured, the fleet start time has passed
/// 3. Admins bypass all permission and visibility restrictions
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated and have view permission for the category
///
/// # Arguments
/// - `state` - Application state containing database, Discord HTTP client, and app URL
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID
/// - `fleet_id` - Fleet ID to fetch
///
/// # Returns
/// - `200 OK` - Fleet details if user has permission to view it
/// - `401 Unauthorized` - User not authenticated
/// - `404 Not Found` - Fleet doesn't exist OR user lacks permission (doesn't leak existence)
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/guilds/{guild_id}/fleets/{fleet_id}",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("fleet_id" = i32, Path, description = "Fleet ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved fleet", body = FleetDto),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 404, description = "Fleet not found or user lacks permission", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_fleet(
    State(state): State<AppState>,
    session: Session,
    Path((_guild_id, fleet_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());

    let fleet = fleet_service
        .get_by_id(fleet_id, user.discord_id, user.admin)
        .await?
        .ok_or_else(|| AppError::NotFound("Fleet not found".to_string()))?;

    Ok((StatusCode::OK, Json(fleet)))
}

/// Get paginated fleets for a guild.
///
/// Returns a paginated list of fleets for the specified guild, filtered by user
/// permissions and visibility rules. Only returns fleets the user has permission
/// to view and respects hidden fleet visibility settings.
///
/// # Visibility Rules
/// Returns fleets filtered by:
/// 1. **Category Permissions**: User must have at least one permission (view, create, or manage) for the category
/// 2. **Time Filter**: Excludes fleets older than 1 hour from current time
/// 3. **Hidden Fleet Visibility**: If a fleet is marked as `hidden`:
///    - User has create OR manage permission for the category, OR
///    - The category's reminder time has elapsed (calculated as fleet_time - ping_reminder), OR
///    - If no reminder is configured, the fleet start time has passed
/// 4. **Admin Override**: Admins bypass all category and visibility filtering
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated
///
/// # Arguments
/// - `state` - Application state containing database, Discord HTTP client, and app URL
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID to fetch fleets for
/// - `pagination` - Pagination parameters (page and per_page)
///
/// # Returns
/// - `200 OK` - Paginated list of visible fleets
/// - `401 Unauthorized` - User not authenticated
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/api/guilds/{guild_id}/fleets",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("page" = Option<u64>, Query, description = "Page number (default: 0)"),
        ("per_page" = Option<u64>, Query, description = "Items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved paginated fleets", body = PaginatedFleetsDto),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_fleets(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleets = fleet_service
        .get_paginated_by_guild(GetPaginatedFleetsByGuildParam {
            guild_id,
            user_id: user.discord_id,
            is_admin: user.admin,
            page: pagination.page,
            per_page: pagination.per_page,
        })
        .await?;

    Ok((StatusCode::OK, Json(fleets)))
}

/// Update a fleet.
///
/// Updates an existing fleet with new time, commander, description, hidden status,
/// or custom field values. Sends Discord notifications for updates. User must be
/// the fleet commander, have manage permission, or be an admin.
///
/// # Authorization
/// User must be:
/// - An admin, OR
/// - The fleet commander, OR
/// - Have manage permission for the fleet's category
///
/// # Visibility
/// The returned updated fleet respects the same visibility rules as GET (see get_fleet).
/// If the fleet is updated to be hidden, the requester can still see it in the response
/// if they have appropriate permissions.
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated and authorized to update the fleet
///
/// # Arguments
/// - `state` - Application state containing database, Discord HTTP client, and app URL
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID
/// - `fleet_id` - Fleet ID to update
/// - `dto` - Fleet update data
///
/// # Returns
/// - `200 OK` - Successfully updated fleet
/// - `401 Unauthorized` - User not authenticated
/// - `403 Forbidden` - User lacks permission to update the fleet
/// - `404 Not Found` - Fleet not found
/// - `400 Bad Request` - Invalid fleet data
/// - `500 Internal Server Error` - Database or Discord API error
#[utoipa::path(
    put,
    path = "/api/guilds/{guild_id}/fleets/{fleet_id}",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("fleet_id" = i32, Path, description = "Fleet ID")
    ),
    request_body = UpdateFleetDto,
    responses(
        (status = 200, description = "Successfully updated fleet", body = FleetDto),
        (status = 400, description = "Invalid fleet data", body = ErrorDto),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 403, description = "User lacks permission to update fleet", body = ErrorDto),
        (status = 404, description = "Fleet not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn update_fleet(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, fleet_id)): Path<(u64, i32)>,
    Json(dto): Json<UpdateFleetDto>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    // Get the fleet to check category and commander
    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleet = fleet_service
        .get_by_id(fleet_id, user.discord_id, user.admin)
        .await?
        .ok_or_else(|| AppError::NotFound("Fleet not found".to_string()))?;

    // Check if user is admin, has manage permission, or is the fleet commander
    let can_manage = if user.admin || user.discord_id == fleet.commander_id {
        true
    } else {
        let permission_repo = UserCategoryPermissionRepository::new(&state.db);
        permission_repo
            .user_can_manage_category(user.discord_id, fleet.category_id)
            .await?
    };

    if !can_manage {
        return Err(AppError::AuthErr(AuthError::AccessDenied(
            user.discord_id,
            "You don't have permission to edit this fleet".to_string(),
        )));
    }

    let updated_fleet = fleet_service
        .update(fleet_id, guild_id, user.discord_id, user.admin, dto)
        .await?;

    Ok((StatusCode::OK, Json(updated_fleet)))
}

/// Delete a fleet.
///
/// Deletes an existing fleet and removes associated Discord messages. User must be
/// the fleet commander, have manage permission, or be an admin.
///
/// # Authorization
/// User must be:
/// - An admin, OR
/// - The fleet commander, OR
/// - Have manage permission for the fleet's category
///
/// # Visibility
/// The fleet must be visible to the user (same rules as GET) before deletion is allowed.
///
/// # Access Control
/// - `LoggedIn` - User must be authenticated and authorized to delete the fleet
///
/// # Arguments
/// - `state` - Application state containing database, Discord HTTP client, and app URL
/// - `session` - User's session for authentication
/// - `guild_id` - Discord guild ID
/// - `fleet_id` - Fleet ID to delete
///
/// # Returns
/// - `204 No Content` - Successfully deleted fleet
/// - `401 Unauthorized` - User not authenticated
/// - `403 Forbidden` - User lacks permission to delete the fleet
/// - `404 Not Found` - Fleet not found or user lacks permission
/// - `500 Internal Server Error` - Database error
#[utoipa::path(
    delete,
    path = "/api/guilds/{guild_id}/fleets/{fleet_id}",
    tag = FLEET_TAG,
    params(
        ("guild_id" = u64, Path, description = "Discord guild ID"),
        ("fleet_id" = i32, Path, description = "Fleet ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted fleet"),
        (status = 401, description = "User not authenticated", body = ErrorDto),
        (status = 403, description = "User lacks permission to delete fleet", body = ErrorDto),
        (status = 404, description = "Fleet not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn delete_fleet(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, fleet_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    // Get the fleet to check category and commander
    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleet = fleet_service
        .get_by_id(fleet_id, user.discord_id, user.admin)
        .await?
        .ok_or_else(|| AppError::NotFound("Fleet not found".to_string()))?;

    // Check if user is admin, has manage permission, or is the fleet commander
    let can_manage = if user.admin || user.discord_id == fleet.commander_id {
        true
    } else {
        let permission_repo = UserCategoryPermissionRepository::new(&state.db);
        permission_repo
            .user_can_manage_category(user.discord_id, fleet.category_id)
            .await?
    };

    if !can_manage {
        return Err(AppError::AuthErr(AuthError::AccessDenied(
            user.discord_id,
            "You don't have permission to delete this fleet".to_string(),
        )));
    }

    let deleted = fleet_service.delete(fleet_id, guild_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Fleet not found".to_string()))
    }
}
