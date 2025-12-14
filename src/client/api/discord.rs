use crate::{
    client::model::error::ApiError,
    model::discord::{PaginatedDiscordGuildChannelsDto, PaginatedDiscordGuildRolesDto},
};

use super::helper::{get, parse_response, send_request};

/// Get paginated roles for a guild
pub async fn get_discord_guild_roles(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedDiscordGuildRolesDto, ApiError> {
    let url = format!(
        "/api/admin/servers/{}/roles?page={}&entries={}",
        guild_id, page, per_page
    );

    let response = send_request(get(&url)).await?;
    parse_response(response).await
}

/// Get paginated channels for a guild
pub async fn get_discord_guild_channels(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedDiscordGuildChannelsDto, ApiError> {
    let url = format!(
        "/api/admin/servers/{}/channels?page={}&entries={}",
        guild_id, page, per_page
    );

    let response = send_request(get(&url)).await?;
    parse_response(response).await
}
