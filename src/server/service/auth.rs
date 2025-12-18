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
    model::user::{UpsertUserParam, User},
    service::discord::UserDiscordGuildRoleService,
    state::OAuth2Client,
};

/// Partial guild information returned from Discord API.
///
/// Contains minimal guild data returned from Discord's user guilds endpoint.
/// Used for identifying which guilds a user belongs to during OAuth flow.
#[derive(Debug, Deserialize)]
pub struct PartialGuild {
    /// Discord guild ID.
    pub id: GuildId,
}

/// Service for Discord OAuth2 authentication and user data synchronization.
///
/// Provides methods for handling Discord OAuth2 login flow, user authentication,
/// and periodic synchronization of user data including guild memberships and roles.
/// Acts as the orchestration layer between Discord API, OAuth2 client, and user repository.
pub struct AuthService<'a> {
    /// Database connection for user operations.
    pub db: &'a DatabaseConnection,
    /// HTTP client for Discord API requests.
    pub http_client: &'a reqwest::Client,
    /// OAuth2 client for Discord authentication flow.
    pub oauth_client: &'a OAuth2Client,
}

impl<'a> AuthService<'a> {
    /// Creates a new AuthService instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    /// - `http_client` - Reference to the HTTP client for Discord API requests
    /// - `oauth_client` - Reference to the configured OAuth2 client
    ///
    /// # Returns
    /// - `AuthService` - New service instance
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

    /// Generates a Discord OAuth2 login URL with CSRF protection.
    ///
    /// Creates an authorization URL that redirects users to Discord's OAuth2 consent screen.
    /// Requests scopes for user identity, guild list, and guild member information. Returns
    /// both the URL and CSRF token for callback validation.
    ///
    /// # Returns
    /// - `(Url, CsrfToken)` - Tuple containing the authorization URL and CSRF state token
    pub fn login_url(&self) -> (Url, CsrfToken) {
        let (authorize_url, csrf_state) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            // Request scope to retrieve user information, guilds, and guild member info
            .add_scope(Scope::new("identify".to_string()))
            .add_scope(Scope::new("guilds".to_string()))
            .add_scope(Scope::new("guilds.members.read".to_string()))
            .url();

        (authorize_url, csrf_state)
    }

    /// Handles OAuth2 callback and authenticates user.
    ///
    /// Exchanges the authorization code for an access token, fetches the user's Discord
    /// information, creates or updates the user record, and syncs their guild/role data
    /// if needed based on timestamp thresholds. Optionally sets admin status for the user.
    ///
    /// # Arguments
    /// - `authorization_code` - OAuth2 authorization code from Discord callback
    /// - `set_admin` - Whether to grant admin privileges to this user
    ///
    /// # Returns
    /// - `Ok(User)` - Authenticated user with updated information
    /// - `Err(AppError::Auth)` - OAuth2 token exchange failed
    /// - `Err(AppError::Network)` - Failed to fetch user data from Discord API
    /// - `Err(AppError::Database)` - Database error during user upsert or sync
    pub async fn callback(
        &self,
        authorization_code: String,
        set_admin: bool,
    ) -> Result<User, AppError> {
        let user_repo = UserRepository::new(self.db);

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
        let new_user = user_repo
            .upsert(UpsertUserParam {
                discord_id: user.id.get().to_string(),
                name: user.name,
                is_admin: admin_update,
            })
            .await?;

        if set_admin {
            tracing::info!("User {} has been set as admin", new_user.name)
        }

        // Sync guilds and roles if needed (based on timestamp threshold)
        self.sync_user_data_if_needed(&new_user, &token).await?;

        Ok(new_user)
    }

    /// Syncs user's role memberships if needed based on timestamp.
    ///
    /// Checks if the user's role data needs refreshing based on a 30-minute threshold.
    /// Guild membership is already tracked via discord_guild_member table from bot events,
    /// so this only syncs Discord roles for logged-in users. Skips sync if recently updated.
    ///
    /// # Arguments
    /// - `user` - User domain model containing sync timestamps
    /// - `token` - OAuth2 access token for Discord API requests
    ///
    /// # Returns
    /// - `Ok(())` - Sync completed or skipped if not needed
    /// - `Err(AppError::Database)` - Database error during sync
    /// - `Err(AppError::Network)` - Failed to fetch data from Discord API
    async fn sync_user_data_if_needed(
        &self,
        user: &User,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let sync_threshold = Duration::minutes(30);

        // DateTimeUtc is already DateTime<Utc>
        let needs_role_sync = now.signed_duration_since(user.last_role_sync_at) > sync_threshold;

        // If sync is not needed, return early
        if !needs_role_sync {
            tracing::debug!(
                "Skipping role sync for user {} (last synced: {})",
                user.discord_id,
                user.last_role_sync_at
            );
            return Ok(());
        }

        // Fetch user guilds for role sync
        let user_guilds = self.fetch_user_guilds(token).await?;

        // Sync roles
        tracing::debug!("Role sync needed for user {}", user.discord_id);
        self.sync_roles(user, token, &user_guilds).await?;

        Ok(())
    }

    /// Syncs user's role memberships across all shared guilds.
    ///
    /// Fetches the user's guild member data for each guild they share with the bot,
    /// updates their role associations in the database, and records the sync timestamp.
    /// Logs warnings for guilds where member data cannot be fetched but continues processing.
    ///
    /// # Arguments
    /// - `user` - User domain model to sync roles for
    /// - `token` - OAuth2 access token for Discord API requests
    /// - `user_guilds` - List of guilds the user belongs to
    ///
    /// # Returns
    /// - `Ok(())` - Role sync completed successfully
    /// - `Err(AppError::Database)` - Database error during sync
    /// - `Err(AppError::InternalError)` - Failed to parse user Discord ID
    async fn sync_roles(
        &self,
        user: &User,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        user_guilds: &[PartialGuild],
    ) -> Result<(), AppError> {
        use crate::server::data::discord::DiscordGuildRepository;
        let guild_repo = DiscordGuildRepository::new(self.db);
        let bot_guilds = guild_repo.get_all().await?;

        let user_id = user
            .discord_id
            .parse::<u64>()
            .map_err(|e| AppError::InternalError(format!("Failed to parse user_id: {}", e)))?;

        let user_role_service = UserDiscordGuildRoleService::new(self.db);
        for guild in user_guilds {
            if bot_guilds
                .iter()
                .any(|bot_guild| bot_guild.guild_id == guild.id.get())
            {
                if let Ok(member) = self.fetch_guild_member(token, guild.id).await {
                    if let Err(e) = user_role_service.sync_user_roles(user_id, &member).await {
                        tracing::warn!(
                            "Failed to sync roles for user {} in guild {}: {:?}",
                            user.discord_id,
                            guild.id,
                            e
                        );
                    }
                }
            }
        }

        let user_repo = UserRepository::new(self.db);
        user_repo.update_role_sync_timestamp(user_id).await?;

        Ok(())
    }

    /// Retrieves a Discord user's information using provided access token.
    ///
    /// Fetches the authenticated user's Discord profile data including their ID, username,
    /// discriminator, and avatar. Uses the Discord API's "@me" endpoint.
    ///
    /// # Arguments
    /// - `token` - OAuth2 access token for the authenticated user
    ///
    /// # Returns
    /// - `Ok(DiscordUser)` - Successfully retrieved user information
    /// - `Err(AppError::Network)` - HTTP request failed or response parsing failed
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

    /// Retrieves a user's member information for a specific guild.
    ///
    /// Fetches the user's Discord Member object for the specified guild using the OAuth token.
    /// The Member object contains the user's roles, nickname, join date, and other guild-specific
    /// data. Used during role synchronization to update local role associations.
    ///
    /// # Arguments
    /// - `token` - OAuth2 access token for the authenticated user
    /// - `guild_id` - Discord's unique identifier for the guild
    ///
    /// # Returns
    /// - `Ok(Member)` - Successfully retrieved member information
    /// - `Err(AppError::Network)` - HTTP request failed or user is not a member of the guild
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

    /// Retrieves a list of guilds the Discord user is a member of.
    ///
    /// Fetches all guilds the authenticated user belongs to from Discord's API. Returns
    /// partial guild information containing only the guild IDs. Used to determine which
    /// guilds to sync role data for.
    ///
    /// # Arguments
    /// - `token` - OAuth2 access token for the authenticated user
    ///
    /// # Returns
    /// - `Ok(Vec<PartialGuild>)` - List of guilds the user is a member of
    /// - `Err(AppError::Network)` - HTTP request failed or response parsing failed
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
