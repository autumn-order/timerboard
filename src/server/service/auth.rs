use chrono::{Duration, Utc};
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
        // Only update admin status if set_admin is true, otherwise preserve existing status
        let admin_update = if set_admin { Some(true) } else { None };
        let new_user = user_repo.upsert(user, admin_update).await?;

        if set_admin {
            tracing::info!("User {} has been set as admin", new_user.name)
        }

        // Sync guilds and roles if needed (based on timestamp threshold)
        self.sync_user_data_if_needed(&new_user, &token).await?;

        Ok(new_user)
    }

    /// Syncs user's guild and role memberships if needed based on timestamps
    ///
    /// Checks if either guild or role sync is needed. If so, fetches the user's guilds
    /// once and performs both syncs as needed to avoid duplicate API calls.
    async fn sync_user_data_if_needed(
        &self,
        user: &entity::user::Model,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let sync_threshold = Duration::minutes(30);

        // DateTimeUtc is already DateTime<Utc>
        let needs_guild_sync = now.signed_duration_since(user.last_guild_sync_at) > sync_threshold;
        let needs_role_sync = now.signed_duration_since(user.last_role_sync_at) > sync_threshold;

        // If neither sync is needed, return early
        if !needs_guild_sync && !needs_role_sync {
            tracing::debug!(
                "Skipping all syncs for user {} (guild: {}, role: {})",
                user.discord_id,
                user.last_guild_sync_at,
                user.last_role_sync_at
            );
            return Ok(());
        }

        // Fetch user guilds once for both syncs
        let user_guilds = self.fetch_user_guilds(token).await?;

        // Sync guilds if needed
        if needs_guild_sync {
            tracing::debug!("Guild sync needed for user {}", user.discord_id);
            self.sync_guilds(user, &user_guilds).await?;
        } else {
            tracing::debug!(
                "Skipping guild sync for user {} (last synced: {})",
                user.discord_id,
                user.last_guild_sync_at
            );
        }

        // Sync roles if needed
        if needs_role_sync {
            tracing::debug!("Role sync needed for user {}", user.discord_id);
            self.sync_roles(user, token, &user_guilds).await?;
        } else {
            tracing::debug!(
                "Skipping role sync for user {} (last synced: {})",
                user.discord_id,
                user.last_role_sync_at
            );
        }

        Ok(())
    }

    /// Syncs user's guild memberships
    async fn sync_guilds(
        &self,
        user: &entity::user::Model,
        user_guilds: &[PartialGuild],
    ) -> Result<(), AppError> {
        let user_guild_ids: Vec<GuildId> = user_guilds.iter().map(|g| g.id).collect();

        let user_guild_service = UserDiscordGuildService::new(self.db);
        user_guild_service
            .sync_user_guilds(user.id, user.discord_id as u64, &user_guild_ids)
            .await?;

        let user_repo = UserRepository::new(self.db);
        user_repo.update_guild_sync_timestamp(user.id).await?;

        Ok(())
    }

    /// Syncs user's role memberships
    async fn sync_roles(
        &self,
        user: &entity::user::Model,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        user_guilds: &[PartialGuild],
    ) -> Result<(), AppError> {
        use crate::server::data::discord::DiscordGuildRepository;
        let guild_repo = DiscordGuildRepository::new(self.db);
        let bot_guilds = guild_repo.get_all().await?;

        let user_role_service = UserDiscordGuildRoleService::new(self.db);
        for guild in user_guilds {
            if bot_guilds.iter().any(|bot_guild| {
                bot_guild
                    .guild_id
                    .parse::<u64>()
                    .map(|id| id == guild.id.get())
                    .unwrap_or(false)
            }) {
                if let Ok(member) = self.fetch_guild_member(token, guild.id).await {
                    if let Err(e) = user_role_service.sync_user_roles(user.id, &member).await {
                        tracing::warn!(
                            "Failed to sync roles for user {} in guild {}: {:?}",
                            user.id,
                            guild.id,
                            e
                        );
                    }
                }
            }
        }

        let user_repo = UserRepository::new(self.db);
        user_repo.update_role_sync_timestamp(user.id).await?;

        Ok(())
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

    /// Retrieves a user's member information for a specific guild
    ///
    /// Fetches the user's Discord Member object for the specified guild using the OAuth token.
    /// The Member object contains the user's roles, nickname, and other guild-specific data.
    ///
    /// # Arguments
    /// - `token`: OAuth access token for the authenticated user
    /// - `guild_id`: Discord's unique identifier for the guild
    ///
    /// # Returns
    /// - `Ok(Member)`: Successfully retrieved member information
    /// - `Err(AppError)`: HTTP request failed or user is not a member of the guild
    async fn fetch_guild_member(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        guild_id: GuildId,
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
}
