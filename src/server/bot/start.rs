use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{ActivityData, Client, Context, EventHandler, GatewayIntents, Guild, Ready};
use serenity::async_trait;

use crate::server::config::Config;
use crate::server::error::AppError;

/// Discord bot event handler
struct Handler {
    db: DatabaseConnection,
}

#[async_trait]
impl EventHandler for Handler {
    /// Called when the bot is ready and connected to Discord
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected to Discord!", ready.user.name);

        ctx.set_activity(Some(ActivityData::custom("Tank Moonman <3")));
    }

    /// Called when a guild becomes available or the bot joins a new guild
    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        tracing::info!("Guild: {} (ID: {})", guild.name, guild.id);
    }
}

/// Starts the Discord bot in a blocking manner
///
/// This function creates and starts the Discord bot client. It should be called from within
/// a tokio::spawn task since it will block until the bot shuts down.
///
/// The bot requires a DISCORD_BOT_TOKEN environment variable to be set.
///
/// # Arguments
/// - `config` - Application configuration
/// - `db` - Database connection for the bot to use
///
/// # Returns
/// - `Ok(())` if the bot starts and runs successfully
/// - `Err(AppError)` if bot initialization or connection fails
pub async fn start_bot(config: &Config, db: DatabaseConnection) -> Result<(), AppError> {
    // Configure gateway intents - what events the bot will receive
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;

    // Create the event handler with database access
    let handler = Handler { db };

    // Build the client
    let mut client = Client::builder(&config.discord_bot_token, intents)
        .event_handler(handler)
        .await?;

    tracing::info!("Starting Discord bot...");

    // Start the bot (this blocks until shutdown)
    client.start().await?;

    Ok(())
}
