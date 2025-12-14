use crate::{client::model::error::ApiError, model::discord::DiscordGuildDto};

use super::helper::{get, parse_response, send_request};

/// Get all Discord guilds that the user has admin access to
pub async fn get_all_discord_guilds() -> Result<Vec<DiscordGuildDto>, ApiError> {
    let response = send_request(get("/api/admin/servers")).await?;
    parse_response(response).await
}

/// Get a specific Discord guild by ID
pub async fn get_discord_guild_by_id(guild_id: u64) -> Result<DiscordGuildDto, ApiError> {
    let url = format!("/api/admin/servers/{}", guild_id);
    let response = send_request(get(&url)).await?;
    parse_response(response).await
}
