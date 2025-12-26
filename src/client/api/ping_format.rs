use crate::{
    client::model::error::ApiError,
    model::ping_format::{
        CreatePingFormatDto, CreatePingFormatFieldDto, PaginatedPingFormatsDto,
        PingFormatFieldType, UpdatePingFormatDto, UpdatePingFormatFieldDto,
    },
};

use super::helper::{
    delete, get, parse_empty_response, parse_response, post, put, send_request, serialize_json,
};

/// Get paginated ping formats for a guild
pub async fn get_ping_formats(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedPingFormatsDto, ApiError> {
    let url = format!(
        "/api/admin/servers/{}/formats?page={}&entries={}",
        guild_id, page, per_page
    );

    let response = send_request(|| get(&url)).await?;
    parse_response(response).await
}

/// Create a new ping format with fields
pub async fn create_ping_format(
    guild_id: u64,
    name: String,
    fields: Vec<(String, i32, PingFormatFieldType, Vec<String>)>, // (name, priority, field_type, default_field_values)
) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/formats", guild_id);
    let payload = CreatePingFormatDto {
        name,
        fields: fields
            .into_iter()
            .map(
                |(name, priority, field_type, default_field_values)| CreatePingFormatFieldDto {
                    name,
                    priority,
                    field_type,
                    default_field_values,
                },
            )
            .collect(),
    };
    let body = serialize_json(&payload)?;

    let response = send_request(|| post(&url).body(body.clone())).await?;
    parse_empty_response(response).await
}

/// Update a ping format and its fields
pub async fn update_ping_format(
    guild_id: u64,
    format_id: i32,
    name: String,
    fields: Vec<(Option<i32>, String, i32, PingFormatFieldType, Vec<String>)>, // (id, name, priority, field_type, default_field_values)
) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/formats/{}", guild_id, format_id);
    let payload = UpdatePingFormatDto {
        name,
        fields: fields
            .into_iter()
            .map(|(id, name, priority, field_type, default_field_values)| {
                UpdatePingFormatFieldDto {
                    id,
                    name,
                    priority,
                    field_type,
                    default_field_values,
                }
            })
            .collect(),
    };
    let body = serialize_json(&payload)?;

    let response = send_request(|| put(&url).body(body.clone())).await?;
    parse_empty_response(response).await
}

/// Delete a ping format
pub async fn delete_ping_format(guild_id: u64, format_id: i32) -> Result<(), ApiError> {
    let url = format!("/api/admin/servers/{}/formats/{}", guild_id, format_id);

    let response = send_request(|| delete(&url)).await?;
    parse_empty_response(response).await
}
