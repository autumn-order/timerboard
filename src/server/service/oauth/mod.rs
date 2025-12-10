//! OAuth2 login with Discord

use crate::server::state::OAuth2Client;

pub mod login;

pub struct DiscordAuthService {
    pub oauth_client: OAuth2Client,
}

impl DiscordAuthService {
    pub fn new(oauth_client: OAuth2Client) -> Self {
        Self { oauth_client }
    }
}
