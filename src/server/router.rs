use axum::{
    routing::{get, post, put},
    Router,
};

use crate::server::{
    controller::{
        admin::add_bot,
        auth::{callback, get_user, login, logout},
        discord::{
            get_all_discord_guilds, get_discord_guild_by_id, get_discord_guild_channels,
            get_discord_guild_roles,
        },
        fleet::{
            create_fleet_category, delete_fleet_category, get_fleet_categories,
            get_fleet_categories_by_ping_format, get_fleet_category_by_id, update_fleet_category,
        },
        ping_format::{
            create_ping_format, delete_ping_format, get_ping_formats, update_ping_format,
        },
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", get(login))
        .route("/api/auth/callback", get(callback))
        .route("/api/auth/logout", get(logout))
        .route("/api/auth/user", get(get_user))
        .route("/api/admin/bot/add", get(add_bot))
        .route("/api/admin/guilds", get(get_all_discord_guilds))
        .route("/api/admin/guild/{guild_id}", get(get_discord_guild_by_id))
        .route(
            "/api/timerboard/{guild_id}/fleet/category",
            post(create_fleet_category).get(get_fleet_categories),
        )
        .route(
            "/api/timerboard/{guild_id}/fleet/category/{fleet_id}",
            get(get_fleet_category_by_id)
                .put(update_fleet_category)
                .delete(delete_fleet_category),
        )
        .route(
            "/api/timerboard/{guild_id}/fleet/category/by-ping-format/{ping_format_id}",
            get(get_fleet_categories_by_ping_format),
        )
        .route(
            "/api/timerboard/{guild_id}/ping/format",
            post(create_ping_format).get(get_ping_formats),
        )
        .route(
            "/api/timerboard/{guild_id}/ping/format/{format_id}",
            put(update_ping_format).delete(delete_ping_format),
        )
        .route(
            "/api/timerboard/{guild_id}/discord/roles",
            get(get_discord_guild_roles),
        )
        .route(
            "/api/timerboard/{guild_id}/discord/channels",
            get(get_discord_guild_channels),
        )
}
