use oauth2::{
    basic::BasicTokenType, AuthorizationCode, EmptyExtraTokenFields, StandardTokenResponse,
    TokenResponse,
};
use serenity::all::User as DiscordUser;

use crate::server::{
    data::user::UserRepository,
    error::{auth::AuthError, AppError},
    service::auth::DiscordAuthService,
};

impl<'a> DiscordAuthService<'a> {
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
