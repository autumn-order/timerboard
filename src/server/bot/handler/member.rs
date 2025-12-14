use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{Context, GuildId, GuildMemberUpdateEvent, Member, User};

use crate::server::data::discord::{DiscordGuildRepository, UserDiscordGuildRepository};
use crate::server::data::user::UserRepository;
use crate::server::service::discord::UserDiscordGuildRoleService;

/// Handles the guild_member_addition event when a member joins a guild
pub async fn handle_guild_member_addition(
    db: &DatabaseConnection,
    _ctx: Context,
    new_member: Member,
) {
    let discord_id = new_member.user.id.get();
    let guild_id = new_member.guild_id.get();

    let user_repo = UserRepository::new(db);
    let guild_repo = DiscordGuildRepository::new(db);
    let user_guild_repo = UserDiscordGuildRepository::new(db);

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
    let guild_id_u64 = match guild.guild_id.parse::<u64>() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to parse guild_id: {:?}", e);
            return;
        }
    };

    if let Err(e) = user_guild_repo.create(user.id, guild_id_u64).await {
        tracing::error!("Failed to create user-guild relationship: {:?}", e);
    } else {
        tracing::info!(
            "User {} joined guild {} - relationship created",
            user.name,
            guild.name
        );
    }
}

/// Handles the guild_member_removal event when a member leaves a guild
pub async fn handle_guild_member_removal(
    db: &DatabaseConnection,
    _ctx: Context,
    guild_id: GuildId,
    user: User,
    _member_data_if_available: Option<Member>,
) {
    let discord_id = user.id.get();
    let guild_id = guild_id.get();

    let user_repo = UserRepository::new(db);
    let guild_repo = DiscordGuildRepository::new(db);
    let user_guild_repo = UserDiscordGuildRepository::new(db);

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
    let guild_id_u64 = match guild.guild_id.parse::<u64>() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to parse guild_id: {:?}", e);
            return;
        }
    };

    if let Err(e) = user_guild_repo.delete(user.id, guild_id_u64).await {
        tracing::error!("Failed to delete user-guild relationship: {:?}", e);
    } else {
        tracing::info!(
            "User {} left guild {} - relationship removed",
            user.name,
            guild.name
        );
    }
}

/// Handles the guild_member_update event when a member is updated in a guild (roles, nickname, etc.)
pub async fn handle_guild_member_update(
    db: &DatabaseConnection,
    _ctx: Context,
    _old: Option<Member>,
    new: Option<Member>,
    _event: GuildMemberUpdateEvent,
) {
    let Some(member) = new else {
        return;
    };

    let discord_id = member.user.id.get();
    let guild_id = member.guild_id.get();

    let user_repo = UserRepository::new(db);

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

    // Sync user's role memberships
    let user_role_service = UserDiscordGuildRoleService::new(db);
    if let Err(e) = user_role_service.sync_user_roles(user.id, &member).await {
        tracing::error!(
            "Failed to sync roles for user {} in guild {}: {:?}",
            user.id,
            guild_id,
            e
        );
    } else {
        tracing::debug!(
            "Synced role memberships for user {} in guild {}",
            user.name,
            guild_id
        );
    }
}
