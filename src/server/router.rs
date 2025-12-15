use axum::{
    routing::{get, put},
    Router,
};

use crate::server::{
    controller::{
        admin::{add_admin, add_bot, get_all_admins, get_all_users, remove_admin},
        auth::{callback, get_user, login, logout},
        category::{
            create_fleet_category, delete_fleet_category, get_fleet_categories,
            get_fleet_categories_by_ping_format, get_fleet_category_by_id, update_fleet_category,
        },
        discord::{
            get_all_discord_guilds, get_discord_guild_by_id, get_discord_guild_channels,
            get_discord_guild_roles,
        },
        fleet::{create_fleet, delete_fleet, get_category_details, get_fleets, get_guild_members},
        ping_format::{
            create_ping_format, delete_ping_format, get_ping_formats, update_ping_format,
        },
        user::{get_user_guilds, get_user_manageable_categories},
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
        .nest("/user", user_router())
        .nest("/guilds", guilds_router())
}

fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
        .route("/user", get(get_user))
}

fn user_router() -> Router<AppState> {
    Router::new().route("/guilds", get(get_user_guilds)).route(
        "/guilds/{guild_id}/manageable-categories",
        get(get_user_manageable_categories),
    )
}

fn guilds_router() -> Router<AppState> {
    Router::new()
        .route("/{guild_id}/members", get(get_guild_members))
        .route(
            "/{guild_id}/categories/{category_id}/details",
            get(get_category_details),
        )
        .route("/{guild_id}/fleets", get(get_fleets).post(create_fleet))
        .route(
            "/{guild_id}/fleets/{fleet_id}",
            axum::routing::delete(delete_fleet),
        )
}

fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/bot/add", get(add_bot))
        .nest("/servers", servers_router())
        .nest("/users", users_router())
        .nest("/admins", admins_router())
}

fn users_router() -> Router<AppState> {
    Router::new().route("/", get(get_all_users))
}

fn admins_router() -> Router<AppState> {
    Router::new().route("/", get(get_all_admins)).route(
        "/{user_id}",
        axum::routing::post(add_admin).delete(remove_admin),
    )
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
