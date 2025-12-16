use super::helper::{
    delete, get, parse_empty_response, parse_response, post, put, send_request, serialize_json,
};
use crate::{
    client::model::error::ApiError,
    model::category::{
        CreateFleetCategoryDto, FleetCategoryDto, PaginatedFleetCategoriesDto,
        UpdateFleetCategoryDto,
    },
};

/// Get paginated fleet categories for a guild
pub async fn get_fleet_categories(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedFleetCategoriesDto, ApiError> {
    let url = format!(
        "/api/admin/servers/{}/categories?page={}&entries={}",
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
    let url = format!("/api/admin/servers/{}/categories/{}", guild_id, category_id);

    let response = send_request(get(&url)).await?;
    parse_response(response).await
}

/// Create a new fleet category
pub async fn create_fleet_category(
    guild_id: u64,
    dto: CreateFleetCategoryDto,
) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/categories", guild_id);
    let body = serialize_json(&dto)?;

    let response = send_request(post(&url).body(body)).await?;
    parse_empty_response(response).await
}

/// Update a fleet category
pub async fn update_fleet_category(
    guild_id: u64,
    category_id: i32,
    dto: UpdateFleetCategoryDto,
) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/categories/{}", guild_id, category_id);
    let body = serialize_json(&dto)?;

    let response = send_request(put(&url).body(body)).await?;
    parse_empty_response(response).await
}

/// Delete a fleet category
pub async fn delete_fleet_category(guild_id: u64, category_id: i32) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/categories/{}", guild_id, category_id);

    let response = send_request(delete(&url)).await?;
    parse_empty_response(response).await
}
