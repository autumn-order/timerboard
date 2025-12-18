//! Application configuration loaded from environment variables.
//!
//! This module provides the `Config` struct which holds all configuration values required
//! for the application to run. Configuration is loaded from environment variables during
//! startup, typically from a `.env` file in development or system environment in production.
//!
//! The configuration includes:
//! - Database connection settings
//! - Application URL for generating links
//! - Discord OAuth2 and bot credentials
//! - Discord API endpoint URLs

use crate::server::error::{config::ConfigError, AppError};

/// Discord OAuth2 authorization endpoint URL.
const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize";

/// Discord OAuth2 token exchange endpoint URL.
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";

/// Application configuration loaded from environment variables.
///
/// All configuration values are loaded during application startup via `from_env()`.
/// Missing or invalid environment variables will cause the application to fail fast
/// with descriptive error messages.
///
/// # Example
/// ```no_run
/// use eve_discord_timerboard::server::config::Config;
///
/// let config = Config::from_env()?;
/// println!("App URL: {}", config.app_url);
/// ```
#[derive(Clone)]
pub struct Config {
    /// Database connection URL (e.g., "sqlite://database.db" or "postgres://...").
    pub database_url: String,

    /// Full application URL including protocol and domain (e.g., "https://example.com").
    /// Used for generating callback URLs and links in Discord embeds.
    pub app_url: String,

    /// Discord application client ID from the Discord Developer Portal.
    pub discord_client_id: String,

    /// Discord application client secret from the Discord Developer Portal.
    pub discord_client_secret: String,

    /// OAuth2 redirect URL registered with Discord (e.g., "https://example.com/api/auth/callback").
    pub discord_redirect_url: String,

    /// Discord bot token for authenticating the bot with Discord's API.
    pub discord_bot_token: String,

    /// Discord OAuth2 authorization endpoint URL.
    pub discord_auth_url: String,

    /// Discord OAuth2 token exchange endpoint URL.
    pub discord_token_url: String,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// Attempts to load all required configuration values from environment variables.
    /// The `app_url` is constructed from `PROTOCOL` and `DOMAIN` environment variables.
    /// All other values are loaded directly from their respective environment variables.
    ///
    /// This function should be called once during application startup. Missing or invalid
    /// environment variables will cause the application to fail immediately with a
    /// descriptive error message.
    ///
    /// # Required Environment Variables
    /// - `DATABASE_URL` - Database connection string
    /// - `PROTOCOL` - Protocol for app URL (e.g., "http" or "https")
    /// - `DOMAIN` - Domain for app URL (e.g., "localhost:8080" or "example.com")
    /// - `DISCORD_CLIENT_ID` - Discord application client ID
    /// - `DISCORD_CLIENT_SECRET` - Discord application client secret
    /// - `DISCORD_REDIRECT_URL` - Discord OAuth2 redirect URL
    /// - `DISCORD_BOT_TOKEN` - Discord bot token
    ///
    /// # Returns
    /// - `Ok(Config)` - Configuration loaded successfully from environment variables
    /// - `Err(ConfigError::MissingEnvVar(_))` - Required environment variable is not set
    pub fn from_env() -> Result<Self, AppError> {
        let protocol = std::env::var("PROTOCOL")
            .map_err(|_| ConfigError::MissingEnvVar("PROTOCOL".to_string()))?;
        let domain = std::env::var("DOMAIN")
            .map_err(|_| ConfigError::MissingEnvVar("DOMAIN".to_string()))?;
        let app_url = format!("{}://{}", protocol, domain);

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?,
            app_url,
            discord_client_id: std::env::var("DISCORD_CLIENT_ID")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_CLIENT_ID".to_string()))?,
            discord_client_secret: std::env::var("DISCORD_CLIENT_SECRET")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_CLIENT_SECRET".to_string()))?,
            discord_redirect_url: std::env::var("DISCORD_REDIRECT_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_REDIRECT_URL".to_string()))?,
            discord_bot_token: std::env::var("DISCORD_BOT_TOKEN")
                .map_err(|_| ConfigError::MissingEnvVar("DISCORD_BOT_TOKEN".to_string()))?,
            discord_auth_url: DISCORD_AUTH_URL.to_string(),
            discord_token_url: DISCORD_TOKEN_URL.to_string(),
        })
    }
}
