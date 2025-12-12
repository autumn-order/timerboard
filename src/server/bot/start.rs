use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{
    ActivityData, Client, Context, EventHandler, GatewayIntents, Guild, Ready, Role, RoleId,
};
use serenity::async_trait;

use crate::server::config::Config;
use crate::server::data::discord::{
    DiscordGuildRepository, DiscordGuildRoleRepository, UserDiscordGuildRepository,
};
use crate::server::data::user::UserRepository;
use crate::server::error::AppError;
use crate::server::service::discord::{DiscordGuildRoleService, UserDiscordGuildService};

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
    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        let guild_id = guild.id.get();
        let guild_roles = guild.roles.clone();
        let cached_members = guild.members.clone();

        tracing::debug!(
            "Guild create event: {} ({}) - member_count: {}, cached_members: {}",
            guild.name,
            guild_id,
            guild.member_count,
            cached_members.len()
        );

        let guild_repo = DiscordGuildRepository::new(&self.db);
        let user_guild_service = UserDiscordGuildService::new(&self.db);

        if let Err(e) = guild_repo.upsert(guild).await {
            tracing::error!("Failed to upsert guild: {:?}", e);
            return;
        }

        let role_service = DiscordGuildRoleService::new(&self.db);

        if let Err(e) = role_service.update_roles(guild_id, &guild_roles).await {
            tracing::error!("Failed to update guild roles: {:?}", e);
        }

        // Fetch members from Discord API since guild.members may not be populated
        // This requires the GUILD_MEMBERS privileged intent
        let member_ids = match ctx
            .http
            .get_guild_members(guild_id.into(), None, None)
            .await
        {
            Ok(members) => {
                let ids: Vec<u64> = members.iter().map(|m| m.user.id.get()).collect();
                tracing::debug!(
                    "Fetched {} members from Discord API for guild {}",
                    ids.len(),
                    guild_id
                );
                ids
            }
            Err(e) => {
                tracing::error!("Failed to fetch guild members from API: {:?}", e);
                // Fallback to cached members if API call fails
                cached_members.keys().map(|id| id.get()).collect()
            }
        };

        // Sync guild members to catch any missed join/leave events while bot was offline
        if let Err(e) = user_guild_service
            .sync_guild_members(guild_id, &member_ids)
            .await
        {
            tracing::error!("Failed to sync guild members: {:?}", e);
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

    /// Called when a member joins a guild
    async fn guild_member_addition(&self, _ctx: Context, new_member: serenity::all::Member) {
        let discord_id = new_member.user.id.get();
        let guild_id = new_member.guild_id.get();

        let user_repo = UserRepository::new(&self.db);
        let guild_repo = DiscordGuildRepository::new(&self.db);
        let user_guild_repo = UserDiscordGuildRepository::new(&self.db);

        // Check if this user is logged into our application
        let Some(user) = (match user_repo.find_by_discord_id(discord_id).await {
            Ok(user) => user,
            Err(e) => {
                tracing::error!("Failed to query user by discord_id: {:?}", e);
                return;
            }
        }) else {
            // User hasn't logged into the app, no need to track
            return;
        };

        // Check if the guild is one our bot is in
        let Some(guild) = (match guild_repo.find_by_guild_id(guild_id).await {
            Ok(guild) => guild,
            Err(e) => {
                tracing::error!("Failed to query guild by guild_id: {:?}", e);
                return;
            }
        }) else {
            // Shouldn't happen since we're receiving the event, but handle it
            tracing::warn!("Received member_add event for unknown guild {}", guild_id);
            return;
        };

        // Create the user-guild relationship
        if let Err(e) = user_guild_repo.create(user.id, guild.id).await {
            tracing::error!("Failed to create user-guild relationship: {:?}", e);
        } else {
            tracing::info!(
                "User {} joined guild {} - relationship created",
                user.name,
                guild.name
            );
        }
    }

    /// Called when a member leaves a guild
    async fn guild_member_removal(
        &self,
        _ctx: Context,
        guild_id: serenity::all::GuildId,
        user: serenity::all::User,
        _member_data_if_available: Option<serenity::all::Member>,
    ) {
        let discord_id = user.id.get();
        let guild_id = guild_id.get();

        let user_repo = UserRepository::new(&self.db);
        let guild_repo = DiscordGuildRepository::new(&self.db);
        let user_guild_repo = UserDiscordGuildRepository::new(&self.db);

        // Check if this user is logged into our application
        let Some(user) = (match user_repo.find_by_discord_id(discord_id).await {
            Ok(user) => user,
            Err(e) => {
                tracing::error!("Failed to query user by discord_id: {:?}", e);
                return;
            }
        }) else {
            // User hasn't logged into the app, no need to track
            return;
        };

        // Check if the guild is one our bot is in
        let Some(guild) = (match guild_repo.find_by_guild_id(guild_id).await {
            Ok(guild) => guild,
            Err(e) => {
                tracing::error!("Failed to query guild by guild_id: {:?}", e);
                return;
            }
        }) else {
            // Shouldn't happen since we're receiving the event, but handle it
            tracing::warn!(
                "Received guild_member_removal event for unknown guild {}",
                guild_id
            );
            return;
        };

        // Delete the user-guild relationship
        if let Err(e) = user_guild_repo.delete(user.id, guild.id).await {
            tracing::error!("Failed to delete user-guild relationship: {:?}", e);
        } else {
            tracing::info!(
                "User {} left guild {} - relationship removed",
                user.name,
                guild.name
            );
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
    // GUILD_MEMBERS is a privileged intent - must be enabled in Discord Developer Portal
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;

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
