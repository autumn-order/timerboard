//! Discord bot service for managing bot invitation and setup.
//!
//! This module provides the `DiscordBotService` for generating OAuth2 URLs that allow
//! administrators to add the Discord bot to their servers. The service configures the
//! appropriate OAuth2 scopes and permissions required for the bot to function properly.

use oauth2::{CsrfToken, Scope};
use serenity::all::Permissions;
use url::Url;

use crate::server::{error::AppError, state::OAuth2Client};

/// Service for managing Discord bot invitation.
///
/// Provides methods for generating OAuth2 authorization URLs that allow server
/// administrators to add the Discord bot to their guilds. The URLs include the
/// necessary OAuth2 scopes and Discord permissions required for bot functionality.
/// Acts as the orchestration layer for bot invitation flow.
pub struct DiscordBotService<'a> {
    /// OAuth2 client for generating authorization URLs.
    pub oauth_client: &'a OAuth2Client,
}

impl<'a> DiscordBotService<'a> {
    /// Creates a new DiscordBotService instance.
    ///
    /// # Arguments
    /// - `oauth_client` - Reference to the configured OAuth2 client
    ///
    /// # Returns
    /// - `DiscordBotService` - New service instance
    pub fn new(oauth_client: &'a OAuth2Client) -> Self {
        Self { oauth_client }
    }

    /// Generates a URL to add Discord bot to a server.
    ///
    /// Creates an OAuth2 authorization URL that redirects administrators to Discord's
    /// bot invitation flow. The URL includes scopes for bot functionality and slash
    /// commands, plus permissions for viewing channels, sending messages, and mentioning
    /// everyone. Returns both the URL and a CSRF token for callback validation.
    ///
    /// # Returns
    /// - `Ok((Url, CsrfToken))` - Bot invitation URL and CSRF state token
    /// - `Err(AppError::Auth)` - OAuth2 URL generation failed
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
