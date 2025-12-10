use crate::server::error::{config::ConfigError, AppError};

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";

pub struct Config {
    pub database_url: String,

    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_redirect_url: String,

    pub discord_auth_url: String,
    pub discord_token_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_CLIENT_ID".to_string()))?,
            discord_client_id: std::env::var("DISCORD_CLIENT_ID")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_CLIENT_SECRET".to_string()))?,
            discord_client_secret: std::env::var("DISCORD_CLIENT_SECRET")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?,
            discord_redirect_url: std::env::var("DISCORD_REDIRECT_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_REDIRECT_URL".to_string()))?,
            discord_auth_url: DISCORD_AUTH_URL.to_string(),
            discord_token_url: DISCORD_TOKEN_URL.to_string(),
        })
    }
}
