//! OAuth2 login with Discord

use sea_orm::DatabaseConnection;

use crate::server::state::OAuth2Client;

pub mod admin;
pub mod callback;
pub mod login;

pub struct DiscordAuthService<'a> {
    pub db: &'a DatabaseConnection,
    pub http_client: &'a reqwest::Client,
    pub oauth_client: &'a OAuth2Client,
}

impl<'a> DiscordAuthService<'a> {
    pub fn new(
        db: &'a DatabaseConnection,
        http_client: &'a reqwest::Client,
        oauth_client: &'a OAuth2Client,
    ) -> Self {
        Self {
            db,
            http_client,
            oauth_client,
        }
    }
}
