use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};

use crate::server::{error::AppError, service::oauth::DiscordAuthService, state::AppState};

pub async fn login(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let login_service = DiscordAuthService::new(state.oauth_client);

    let (url, csrf_token) = login_service.login_url();

    // TODO: Insert CSRF state to session

    Ok(Redirect::temporary(&url.to_string()))
}
