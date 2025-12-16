use oauth2::{CsrfToken, Scope};
use serenity::all::Permissions;
use url::Url;

use crate::server::{error::AppError, state::OAuth2Client};

pub struct DiscordBotService<'a> {
    pub oauth_client: &'a OAuth2Client,
}

impl<'a> DiscordBotService<'a> {
    pub fn new(oauth_client: &'a OAuth2Client) -> Self {
        Self { oauth_client }
    }

    /// Generates a URL to add Discord bot to server
    pub async fn bot_url(&self) -> Result<(Url, CsrfToken), AppError> {
        let (mut authorize_url, csrf_token) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            // Request scope to add bot and slash commands
            .add_scope(Scope::new("bot".to_string()))
            .add_scope(Scope::new("applications.commands".to_string()))
            .url();

        let permissions =
            Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::MENTION_EVERYONE;

        authorize_url
            .query_pairs_mut()
            .append_pair("permissions", &permissions.bits().to_string());

        Ok((authorize_url, csrf_token))
    }
}
