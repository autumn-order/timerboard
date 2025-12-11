use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{
    ActivityData, Client, Context, EventHandler, GatewayIntents, Guild, Ready, Role, RoleId,
};
use serenity::async_trait;

use crate::server::config::Config;
use crate::server::data::discord::{DiscordGuildRepository, DiscordGuildRoleRepository};
use crate::server::error::AppError;
use crate::server::service::discord::DiscordGuildRoleService;

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
        let guild_id = guild.id.get();
        let guild_roles = guild.roles.clone();

        let guild_repo = DiscordGuildRepository::new(&self.db);

        if let Err(e) = guild_repo.upsert(guild).await {
            tracing::error!("Failed to upsert guild: {:?}", e);
            return;
        }

        let role_service = DiscordGuildRoleService::new(&self.db);

        if let Err(e) = role_service.update_roles(guild_id, &guild_roles).await {
            tracing::error!("Failed to update guild roles: {:?}", e);
        }
    }

    /// Called when a role is created in a guild
    async fn guild_role_create(&self, _ctx: Context, new: Role) {
        let guild_id = new.guild_id.get();
        let role_repo = DiscordGuildRoleRepository::new(&self.db);

        if let Err(e) = role_repo.upsert(guild_id, &new).await {
            tracing::error!("Failed to upsert new role: {:?}", e);
        } else {
            tracing::info!("Created role {} in guild {}", new.name, guild_id);
        }
    }

    /// Called when a role is updated in a guild
    async fn guild_role_update(&self, _ctx: Context, _old: Option<Role>, new: Role) {
        let guild_id = new.guild_id.get();
        let role_repo = DiscordGuildRoleRepository::new(&self.db);

        if let Err(e) = role_repo.upsert(guild_id, &new).await {
            tracing::error!("Failed to upsert updated role: {:?}", e);
        } else {
            tracing::info!("Updated role {} in guild {}", new.name, guild_id);
        }
    }

    /// Called when a role is deleted from a guild
    async fn guild_role_delete(
        &self,
        _ctx: Context,
        guild_id: serenity::all::GuildId,
        removed_role_id: RoleId,
        _removed_role_data_if_in_cache: Option<Role>,
    ) {
        let role_repo = DiscordGuildRoleRepository::new(&self.db);

        if let Err(e) = role_repo.delete(removed_role_id.get()).await {
            tracing::error!("Failed to delete role: {:?}", e);
        } else {
            tracing::info!("Deleted role {} from guild {}", removed_role_id, guild_id);
        }
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
