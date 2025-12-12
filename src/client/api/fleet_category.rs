use crate::{
    client::model::error::ApiError,
    model::fleet::{CreateFleetCategoryDto, PaginatedFleetCategoriesDto, UpdateFleetCategoryDto},
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

/// Create a new fleet category
pub async fn create_fleet_category(guild_id: u64, name: String) -> Result<(), ApiError> {
    let url = format!("/api/timerboard/{}/fleet/category", guild_id);
    let payload = CreateFleetCategoryDto { name };
    let body = serialize_json(&payload)?;

    let response = send_request(post(&url).body(body)).await?;
    parse_empty_response(response).await
}

/// Update a fleet category
pub async fn update_fleet_category(
    guild_id: u64,
    category_id: i32,
    name: String,
) -> Result<(), ApiError> {
    let url = format!(
        "/api/timerboard/{}/fleet/category/{}",
        guild_id, category_id
    );
    let payload = UpdateFleetCategoryDto { name };
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
