use axum::{routing::get, Router};

use crate::server::{
    controller::auth::{callback, login},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", get(login))
        .route("/api/auth/callback", get(callback))
}
