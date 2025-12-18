use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    model::{
        category::{FleetCategoryDetailsDto, PingFormatFieldDto},
        discord::DiscordGuildMemberDto,
        fleet::{CreateFleetDto, UpdateFleetDto},
    },
    server::{
        data::category::FleetCategoryRepository,
        error::{auth::AuthError, AppError},
        middleware::auth::{AuthGuard, Permission},
        service::fleet::FleetService,
        state::AppState,
    },
};

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

/// GET /api/guilds/{guild_id}/categories/{category_id}/details
/// Get category details including ping format fields for fleet creation
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
        .get_category_details(category_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    // Get ping format fields
    let fields = entity::prelude::PingFormatField::find()
        .filter(
            entity::ping_format_field::Column::PingFormatId
                .eq(category_with_relations.category.ping_format_id),
        )
        .order_by_asc(entity::ping_format_field::Column::Priority)
        .all(&state.db)
        .await?
        .into_iter()
        .map(|f| PingFormatFieldDto {
            id: f.id,
            name: f.name,
            priority: f.priority,
            default_value: f.default_value,
        })
        .collect();

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

/// GET /api/guilds/{guild_id}/members
/// Get all members of a guild for FC selection
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

/// POST /api/guilds/{guild_id}/fleets
/// Create a new fleet
pub async fn create_fleet(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Json(dto): Json<CreateFleetDto>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session)
        .require(&[Permission::CategoryCreate(guild_id, dto.category_id)])
        .await?;

    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleet = fleet_service.create(dto, user.admin).await?;

    Ok((StatusCode::CREATED, Json(fleet)))
}

/// GET /api/guilds/{guild_id}/fleets/{fleet_id}
/// Get fleet details by ID
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
/// # Returns
/// - 200 OK with FleetDto if fleet exists and user has permission to view it
/// - 404 Not Found if fleet doesn't exist OR user lacks permission (doesn't leak existence)
pub async fn get_fleet(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, fleet_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    let user_id = user
        .discord_id
        .parse::<u64>()
        .map_err(|e| AppError::InternalError(format!("Invalid user discord_id: {}", e)))?;

    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleet = fleet_service
        .get_by_id(fleet_id, guild_id, user_id, user.admin)
        .await?
        .ok_or_else(|| AppError::NotFound("Fleet not found".to_string()))?;

    Ok((StatusCode::OK, Json(fleet)))
}

/// GET /api/guilds/{guild_id}/fleets
/// Get paginated fleets for a guild
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
/// # Returns
/// - 200 OK with PaginatedFleetsDto containing visible fleets
pub async fn get_fleets(
    State(state): State<AppState>,
    session: Session,
    Path(guild_id): Path<u64>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    let user_id = user
        .discord_id
        .parse::<u64>()
        .map_err(|e| AppError::InternalError(format!("Invalid user discord_id: {}", e)))?;

    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleets = fleet_service
        .get_paginated_by_guild(
            guild_id,
            user_id,
            user.admin,
            pagination.page,
            pagination.per_page,
        )
        .await?;

    Ok((StatusCode::OK, Json(fleets)))
}

/// PUT /api/guilds/{guild_id}/fleets/{fleet_id}
/// Update a fleet (requires manage permission or being the fleet commander)
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
pub async fn update_fleet(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, fleet_id)): Path<(u64, i32)>,
    Json(dto): Json<UpdateFleetDto>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    let user_id = user
        .discord_id
        .parse::<u64>()
        .map_err(|e| AppError::InternalError(format!("Invalid user discord_id: {}", e)))?;

    // Get the fleet to check category and commander
    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleet = fleet_service
        .get_by_id(fleet_id, guild_id, user_id, user.admin)
        .await?
        .ok_or_else(|| AppError::NotFound("Fleet not found".to_string()))?;

    // Check if user is admin, has manage permission, or is the fleet commander
    let can_manage = if user.admin || user_id == fleet.commander_id {
        true
    } else {
        let category_repo = FleetCategoryRepository::new(&state.db);
        category_repo
            .user_can_manage_category(user_id, guild_id, fleet.category_id)
            .await?
    };

    if !can_manage {
        return Err(AppError::AuthErr(AuthError::AccessDenied(
            user_id,
            "You don't have permission to edit this fleet".to_string(),
        )));
    }

    let updated_fleet = fleet_service
        .update(fleet_id, guild_id, user_id, user.admin, dto)
        .await?;

    Ok((StatusCode::OK, Json(updated_fleet)))
}

/// DELETE /api/guilds/{guild_id}/fleets/{fleet_id}
/// Delete a fleet (requires manage permission or being the fleet commander)
///
/// # Authorization
/// User must be:
/// - An admin, OR
/// - The fleet commander, OR
/// - Have manage permission for the fleet's category
///
/// # Visibility
/// The fleet must be visible to the user (same rules as GET) before deletion is allowed.
pub async fn delete_fleet(
    State(state): State<AppState>,
    session: Session,
    Path((guild_id, fleet_id)): Path<(u64, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthGuard::new(&state.db, &session).require(&[]).await?;

    let user_id = user
        .discord_id
        .parse::<u64>()
        .map_err(|e| AppError::InternalError(format!("Invalid user discord_id: {}", e)))?;

    // Get the fleet to check category and commander
    let fleet_service =
        FleetService::new(&state.db, state.discord_http.clone(), state.app_url.clone());
    let fleet = fleet_service
        .get_by_id(fleet_id, guild_id, user_id, user.admin)
        .await?
        .ok_or_else(|| AppError::NotFound("Fleet not found".to_string()))?;

    // Check if user is admin, has manage permission, or is the fleet commander
    let can_manage = if user.admin || user_id == fleet.commander_id {
        true
    } else {
        let category_repo = FleetCategoryRepository::new(&state.db);
        category_repo
            .user_can_manage_category(user_id, guild_id, fleet.category_id)
            .await?
    };

    if !can_manage {
        return Err(AppError::AuthErr(AuthError::AccessDenied(
            user_id,
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
