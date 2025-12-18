//! Discord bot initialization and startup.
//!
//! This module provides functions for initializing and starting the Discord bot client.
//! The bot is initialized during server startup to extract the HTTP client for use by
//! other services, then started in a separate tokio task to handle Discord events.
//!
//! The bot requires the following gateway intents:
//! - `GUILDS` - Access to guild information
//! - `GUILD_MESSAGES` - Access to guild messages (for commands/interactions)
//! - `GUILD_MEMBERS` - Access to guild member information (privileged intent)
//!
//! Note: `GUILD_MEMBERS` is a privileged intent and must be enabled in the Discord
//! Developer Portal for the bot application.

use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{Client, GatewayIntents};
use serenity::http::Http;
use std::sync::Arc;

use crate::server::config::Config;
use crate::server::error::AppError;

use super::handler::Handler;

/// Initializes the Discord bot client and returns the HTTP client.
///
/// Creates and configures the Discord bot client with the necessary gateway intents
/// and event handlers. The bot is initialized but not started, allowing the HTTP
/// client to be extracted and shared with other parts of the application (such as
/// the fleet notification service) before the bot begins processing events.
///
/// The bot is configured with:
/// - Gateway intents for guilds, messages, and member information
/// - Event handler with database access for processing Discord events
/// - Authentication using the bot token from configuration
///
/// # Arguments
/// - `config` - Application configuration containing the Discord bot token
/// - `db` - Database connection for the bot to use in event handlers
///
/// # Returns
/// - `Ok((Client, Arc<Http>))` - The bot client and HTTP client for Discord API operations
/// - `Err(AppError::DiscordError)` - Failed to build the Discord client
pub async fn init_bot(
    config: &Config,
    db: DatabaseConnection,
) -> Result<(Client, Arc<Http>), AppError> {
    tracing::info!("Initializing Discord bot client");

    // Configure gateway intents - what events the bot will receive
    // GUILD_MEMBERS is a privileged intent - must be enabled in Discord Developer Portal
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;

    // Create the event handler with database access
    let handler = Handler::new(db);

    // Build the client
    let client = Client::builder(&config.discord_bot_token, intents)
        .event_handler(handler)
        .await?;

    // Clone the HTTP client to share with the rest of the app
    let http = client.http.clone();

    tracing::info!("Discord bot client initialized successfully");

    Ok((client, http))
}

/// Starts the Discord bot connection (blocking until shutdown).
///
/// Establishes the WebSocket connection to Discord's gateway and begins processing
/// events. This function blocks until the bot is shut down or encounters a fatal error.
/// It should be called from within a `tokio::spawn` task to avoid blocking the main
/// server startup.
///
/// The bot will:
/// - Connect to Discord's gateway
/// - Authenticate with the bot token
/// - Begin receiving and processing events via the configured event handler
/// - Maintain the connection with automatic reconnection on temporary failures
///
/// # Arguments
/// - `client` - The Discord bot client to start
///
/// # Returns
/// - `Ok(())` - The bot shut down gracefully
/// - `Err(AppError::DiscordError)` - Failed to connect or maintain connection to Discord
pub async fn start_bot(mut client: Client) -> Result<(), AppError> {
    tracing::info!("Starting Discord bot connection");

    // Start the bot (this blocks until shutdown)
    client.start().await?;

    tracing::warn!("Discord bot connection closed");

    Ok(())
}
