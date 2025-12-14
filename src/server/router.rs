use axum::{
    routing::{get, put},
    Router,
};

use crate::server::{
    controller::{
        admin::add_bot,
        auth::{callback, get_user, login, logout},
        category::{
            create_fleet_category, delete_fleet_category, get_fleet_categories,
            get_fleet_categories_by_ping_format, get_fleet_category_by_id, update_fleet_category,
        },
        discord::{
            get_all_discord_guilds, get_discord_guild_by_id, get_discord_guild_channels,
            get_discord_guild_roles,
        },
        ping_format::{
            create_ping_format, delete_ping_format, get_ping_formats, update_ping_format,
        },
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().nest("/api", api_router())
}

fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_router())
        .nest("/admin", admin_router())
}

fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
        .route("/user", get(get_user))
}

fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/bot/add", get(add_bot))
        .nest("/servers", servers_router())
}

fn servers_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_discord_guilds))
        .route("/{guild_id}", get(get_discord_guild_by_id))
        .nest("/{guild_id}/categories", server_categories_router())
        .nest("/{guild_id}/formats", server_formats_router())
        .nest("/{guild_id}/roles", server_roles_router())
        .nest("/{guild_id}/channels", server_channels_router())
}

fn server_categories_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_fleet_categories).post(create_fleet_category))
        .route(
            "/{category_id}",
            get(get_fleet_category_by_id)
                .put(update_fleet_category)
                .delete(delete_fleet_category),
        )
}

fn server_formats_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_ping_formats).post(create_ping_format))
        .route(
            "/{format_id}",
            put(update_ping_format).delete(delete_ping_format),
        )
        .route(
            "/{format_id}/categories",
            get(get_fleet_categories_by_ping_format),
        )
}

fn server_roles_router() -> Router<AppState> {
    Router::new().route("/", get(get_discord_guild_roles))
}

fn server_channels_router() -> Router<AppState> {
    Router::new().route("/", get(get_discord_guild_channels))
}
