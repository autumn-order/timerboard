use sea_orm::DatabaseConnection;

use crate::server::startup::OAuth2Client;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub oauth_client: OAuth2Client,
}
