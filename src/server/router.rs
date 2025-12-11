use axum::{routing::get, Router};

use crate::server::{
    controller::auth::{callback, get_user, login, logout},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", get(login))
        .route("/api/auth/callback", get(callback))
        .route("/api/auth/logout", get(logout))
        .route("/api/auth/user", get(get_user))
}
