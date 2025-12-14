use chrono::Duration;

use crate::{
    client::model::error::ApiError,
    model::category::{
        CreateFleetCategoryDto, FleetCategoryAccessRoleDto, FleetCategoryChannelDto,
        FleetCategoryDto, FleetCategoryPingRoleDto, PaginatedFleetCategoriesDto,
        UpdateFleetCategoryDto,
    },
};

use super::helper::{
    delete, get, parse_empty_response, parse_response, post, put, send_request, serialize_json,
};

/// Get paginated fleet categories for a guild
pub async fn get_fleet_categories(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedFleetCategoriesDto, ApiError> {
    let url = format!(
        "/api/timerboard/{}/fleet/category?page={}&entries={}",
        guild_id, page, per_page
    );

    let response = send_request(get(&url)).await?;
    parse_response(response).await
}

/// Get a specific fleet category by ID
pub async fn get_fleet_category_by_id(
    guild_id: u64,
    category_id: i32,
) -> Result<FleetCategoryDto, ApiError> {
    let url = format!(
        "/api/timerboard/{}/fleet/category/{}",
        guild_id, category_id
    );

    let response = send_request(get(&url)).await?;
    parse_response(response).await
}

/// Create a new fleet category
pub async fn create_fleet_category(
    guild_id: u64,
    ping_format_id: i32,
    name: String,
    ping_cooldown: Option<Duration>,
    ping_reminder: Option<Duration>,
    max_pre_ping: Option<Duration>,
    access_roles: Vec<FleetCategoryAccessRoleDto>,
    ping_roles: Vec<FleetCategoryPingRoleDto>,
    channels: Vec<FleetCategoryChannelDto>,
) -> Result<(), ApiError> {
    let url = format!("/api/timerboard/{}/fleet/category", guild_id);
    let payload = CreateFleetCategoryDto {
        ping_format_id,
        name,
        ping_lead_time: ping_cooldown,
        ping_reminder,
        max_pre_ping,
        access_roles,
        ping_roles,
        channels,
    };
    let body = serialize_json(&payload)?;

    let response = send_request(post(&url).body(body)).await?;
    parse_empty_response(response).await
}

/// Update a fleet category
pub async fn update_fleet_category(
    guild_id: u64,
    category_id: i32,
    ping_format_id: i32,
    name: String,
    ping_cooldown: Option<Duration>,
    ping_reminder: Option<Duration>,
    max_pre_ping: Option<Duration>,
    access_roles: Vec<FleetCategoryAccessRoleDto>,
    ping_roles: Vec<FleetCategoryPingRoleDto>,
    channels: Vec<FleetCategoryChannelDto>,
) -> Result<(), ApiError> {
    let url = format!(
        "/api/timerboard/{}/fleet/category/{}",
        guild_id, category_id
    );
    let payload = UpdateFleetCategoryDto {
        ping_format_id,
        name,
        ping_lead_time: ping_cooldown,
        ping_reminder,
        max_pre_ping,
        access_roles,
        ping_roles,
        channels,
    };
    let body = serialize_json(&payload)?;

    let response = send_request(put(&url).body(body)).await?;
    parse_empty_response(response).await
}

/// Delete a fleet category
pub async fn delete_fleet_category(guild_id: u64, category_id: i32) -> Result<(), ApiError> {
    let url = format!(
        "/api/timerboard/{}/fleet/category/{}",
        guild_id, category_id
    );

    let response = send_request(delete(&url)).await?;
    parse_empty_response(response).await
}

/// Get fleet categories by ping format ID
pub async fn get_fleet_categories_by_ping_format(
    guild_id: u64,
    ping_format_id: i32,
) -> Result<Vec<FleetCategoryDto>, ApiError> {
    let url = format!(
        "/api/timerboard/{}/fleet/category/by-ping-format/{}",
        guild_id, ping_format_id
    );

    let response = send_request(get(&url)).await?;
    parse_response(response).await
}
