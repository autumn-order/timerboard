use crate::{
    client::model::error::ApiError,
    model::{
        category::FleetCategoryDetailsDto,
        discord::DiscordGuildMemberDto,
        fleet::{CreateFleetDto, FleetDto, PaginatedFleetsDto, UpdateFleetDto},
    },
};

use super::helper::{
    delete, get, parse_empty_response, parse_response, post, put, send_request, serialize_json,
};

/// GET /api/guilds/{guild_id}/categories/{category_id}/details
/// Get category details including ping format fields for fleet creation
pub async fn get_category_details(
    guild_id: u64,
    category_id: i32,
) -> Result<FleetCategoryDetailsDto, ApiError> {
    let url = format!(
        "/api/guilds/{}/categories/{}/details",
        guild_id, category_id
    );
    let request = get(&url);
    let response = send_request(request).await?;
    parse_response(response).await
}

/// GET /api/guilds/{guild_id}/members
/// Get all members of a guild for FC selection
pub async fn get_guild_members(guild_id: u64) -> Result<Vec<DiscordGuildMemberDto>, ApiError> {
    let url = format!("/api/guilds/{}/members", guild_id);
    let request = get(&url);
    let response = send_request(request).await?;
    parse_response(response).await
}

/// POST /api/guilds/{guild_id}/fleets
/// Create a new fleet
pub async fn create_fleet(guild_id: u64, dto: CreateFleetDto) -> Result<FleetDto, ApiError> {
    let url = format!("/api/guilds/{}/fleets", guild_id);
    let body = serialize_json(&dto)?;
    let response = send_request(post(&url).body(body)).await?;
    parse_response(response).await
}

/// GET /api/guilds/{guild_id}/fleets
/// Get paginated fleets for a guild
pub async fn get_fleets(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedFleetsDto, ApiError> {
    let url = format!(
        "/api/guilds/{}/fleets?page={}&per_page={}",
        guild_id, page, per_page
    );
    let request = get(&url);
    let response = send_request(request).await?;
    parse_response(response).await
}

/// GET /api/guilds/{guild_id}/fleets/{fleet_id}
/// Get fleet details by ID
pub async fn get_fleet(guild_id: u64, fleet_id: i32) -> Result<FleetDto, ApiError> {
    let url = format!("/api/guilds/{}/fleets/{}", guild_id, fleet_id);
    let request = get(&url);
    let response = send_request(request).await?;
    parse_response(response).await
}

/// PUT /api/guilds/{guild_id}/fleets/{fleet_id}
/// Update a fleet
pub async fn update_fleet(
    guild_id: u64,
    fleet_id: i32,
    dto: UpdateFleetDto,
) -> Result<FleetDto, ApiError> {
    let url = format!("/api/guilds/{}/fleets/{}", guild_id, fleet_id);
    let body = serialize_json(&dto)?;
    let response = send_request(put(&url).body(body)).await?;
    parse_response(response).await
}

/// DELETE /api/guilds/{guild_id}/fleets/{fleet_id}
/// Delete a fleet
pub async fn delete_fleet(guild_id: u64, fleet_id: i32) -> Result<(), ApiError> {
    let url = format!("/api/guilds/{}/fleets/{}", guild_id, fleet_id);
    let response = send_request(delete(&url)).await?;
    parse_empty_response(response).await
}
