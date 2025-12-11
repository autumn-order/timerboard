use axum::{routing::get, Router};

use crate::server::{
    controller::{
        admin::add_bot,
        auth::{callback, get_user, login, logout},
        discord::get_all_discord_guilds,
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
}
