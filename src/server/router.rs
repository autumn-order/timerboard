use axum::{
    routing::{get, post, put},
    Router,
};

use crate::server::{
    controller::{
        admin::add_bot,
        auth::{callback, get_user, login, logout},
        discord::{get_all_discord_guilds, get_discord_guild_by_id},
        fleet::{
            create_fleet_category, delete_fleet_category, get_fleet_categories,
            update_fleet_category,
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
        .route("/api/admin/discord/guilds", get(get_all_discord_guilds))
        .route(
            "/api/admin/discord/guilds/{guild_id}",
            get(get_discord_guild_by_id),
        )
        .route(
            "/api/timerboard/{guild_id}/fleet/category",
            post(create_fleet_category).get(get_fleet_categories),
        )
        .route(
            "/api/timerboard/{guild_id}/fleet/category/{fleet_id}",
            put(update_fleet_category).delete(delete_fleet_category),
        )
}
