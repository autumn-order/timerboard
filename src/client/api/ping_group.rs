use crate::{
    client::model::error::ApiError,
    model::ping_group::{CreatePingGroupDto, PaginatedPingGroupsDto, UpdatePingGroupDto},
};

use super::helper::{
    delete, get, parse_empty_response, parse_response, post, put, send_request, serialize_json,
};

pub async fn create_ping_group(guild_id: u64, payload: CreatePingGroupDto) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/ping-group", guild_id);
    let body = serialize_json(&payload)?;

    let response = send_request(|| post(&url).body(body.clone())).await?;
    parse_empty_response(response).await
}

pub async fn get_paginated_ping_groups(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedPingGroupsDto, ApiError> {
    let url = format!(
        "/api/admin/servers/{}/ping-groups?page={}&entries={}",
        guild_id, page, per_page
    );

    let response = send_request(|| get(&url)).await?;
    parse_response(response).await
}

pub async fn update_ping_group(
    guild_id: u64,
    id: i32,
    payload: UpdatePingGroupDto,
) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/ping-group/{}", guild_id, id);
    let body = serialize_json(&payload)?;

    let response = send_request(|| put(&url).body(body.clone())).await?;
    parse_empty_response(response).await
}

pub async fn delete_ping_group(guild_id: u64, id: i32) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/ping-group/{}", guild_id, id);

    let response = send_request(|| delete(&url)).await?;
    parse_empty_response(response).await
}
