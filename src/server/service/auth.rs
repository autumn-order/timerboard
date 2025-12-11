use oauth2::{
    basic::BasicTokenType, AuthorizationCode, CsrfToken, EmptyExtraTokenFields, Scope,
    StandardTokenResponse, TokenResponse,
};
use sea_orm::DatabaseConnection;
use serenity::all::User as DiscordUser;
use url::Url;

use crate::server::{
    data::user::UserRepository,
    error::{auth::AuthError, AppError},
    state::OAuth2Client,
};

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

    pub fn login_url(&self) -> (Url, CsrfToken) {
        let (authorize_url, csrf_state) = self
            .oauth_client
            .authorize_url(|| CsrfToken::new_random())
            // Request scope to retrieve user information without email
            .add_scope(Scope::new("identify".to_string()))
            .url();

        (authorize_url, csrf_state)
    }

    pub async fn callback(
        &self,
        authorization_code: String,
        is_admin: bool,
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
        let new_user = user_repo.upsert(user, is_admin).await?;

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
}
