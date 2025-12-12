use dioxus_logger::tracing;
use oauth2::{
    basic::BasicTokenType, AuthorizationCode, CsrfToken, EmptyExtraTokenFields, Scope,
    StandardTokenResponse, TokenResponse,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serenity::all::{GuildId, User as DiscordUser};
use url::Url;

use crate::server::{
    data::user::UserRepository,
    error::{auth::AuthError, AppError},
    service::discord::{UserDiscordGuildRoleService, UserDiscordGuildService},
    state::OAuth2Client,
};

/// Partial guild information returned from Discord API
#[derive(Debug, Deserialize)]
pub struct PartialGuild {
    pub id: GuildId,
}

pub struct AuthService<'a> {
    pub db: &'a DatabaseConnection,
    pub http_client: &'a reqwest::Client,
    pub oauth_client: &'a OAuth2Client,
}

impl<'a> AuthService<'a> {
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

    pub fn login_url(&self) -> (Url, CsrfToken) {
        let (authorize_url, csrf_state) = self
            .oauth_client
            .authorize_url(|| CsrfToken::new_random())
            // Request scope to retrieve user information, guilds, and guild member info
            .add_scope(Scope::new("identify".to_string()))
            .add_scope(Scope::new("guilds".to_string()))
            .add_scope(Scope::new("guilds.members.read".to_string()))
            .url();

        (authorize_url, csrf_state)
    }

    pub async fn callback(
        &self,
        authorization_code: String,
        set_admin: bool,
    ) -> Result<entity::user::Model, AppError> {
        let user_repo = UserRepository::new(&self.db);

        let auth_code = AuthorizationCode::new(authorization_code);

        let token = self
            .oauth_client
            .exchange_code(auth_code)
            .request_async(self.http_client)
            .await
            .map_err(AuthError::from)?;

        let user = self.fetch_discord_user(&token).await?;
        let new_user = user_repo.upsert(user, set_admin).await?;

        if set_admin {
            tracing::info!("User {} has been set as admin", new_user.name)
        }

        // Fetch and sync user's Discord guilds
        let user_guilds = self.fetch_user_guilds(&token).await?;
        let user_guild_ids: Vec<GuildId> = user_guilds.iter().map(|g| g.id).collect();

        let user_guild_service = UserDiscordGuildService::new(self.db);
        user_guild_service
            .sync_user_guilds(new_user.id, &user_guild_ids)
            .await?;

        // Fetch and sync user's role memberships for each guild
        let user_role_service = UserDiscordGuildRoleService::new(self.db);
        for guild in &user_guilds {
            if let Ok(member) = self
                .fetch_guild_member(&token, guild.id, new_user.discord_id as u64)
                .await
            {
                if let Err(e) = user_role_service
                    .sync_user_roles(new_user.id, &member)
                    .await
                {
                    tracing::warn!(
                        "Failed to sync roles for user {} in guild {}: {:?}",
                        new_user.id,
                        guild.id,
                        e
                    );
                }
            }
        }

        Ok(new_user)
    }

    /// Retrieves a Discord user's information using provided access token
    async fn fetch_discord_user(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<DiscordUser, AppError> {
        let access_token = token.access_token().secret();

        let user_info = self
            .http_client
            .get("https://discord.com/api/users/@me")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<DiscordUser>()
            .await?;

        Ok(user_info)
    }

    /// Retrieves a list of guilds the Discord user is a member of
    pub async fn fetch_user_guilds(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<Vec<PartialGuild>, AppError> {
        let access_token = token.access_token().secret();

        let guilds = self
            .http_client
            .get("https://discord.com/api/users/@me/guilds")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<Vec<PartialGuild>>()
            .await?;

        Ok(guilds)
    }

    /// Retrieves a user's member information for a specific guild
    ///
    /// Fetches the user's Discord Member object for the specified guild using the OAuth token.
    /// The Member object contains the user's roles, nickname, and other guild-specific data.
    ///
    /// # Arguments
    /// - `token`: OAuth access token for the authenticated user
    /// - `guild_id`: Discord's unique identifier for the guild
    /// - `user_id`: Discord's unique identifier for the user
    ///
    /// # Returns
    /// - `Ok(Member)`: Successfully retrieved member information
    /// - `Err(AppError)`: HTTP request failed or user is not a member of the guild
    async fn fetch_guild_member(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        guild_id: GuildId,
        user_id: u64,
    ) -> Result<serenity::all::Member, AppError> {
        let access_token = token.access_token().secret();

        let member = self
            .http_client
            .get(format!(
                "https://discord.com/api/users/@me/guilds/{}/member",
                guild_id.get()
            ))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json::<serenity::all::Member>()
            .await?;

        Ok(member)
    }
}
