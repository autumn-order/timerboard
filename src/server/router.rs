use axum::{routing::get, Router};

use crate::server::{controller::auth::login, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/auth/login", get(login))
}
