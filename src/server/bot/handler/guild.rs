//! Guild event handlers for Discord guild synchronization.
//!
//! This module handles the `guild_create` event which fires when a guild becomes
//! available to the bot. This occurs in several scenarios:
//! - On bot startup for each guild the bot is already in
//! - When the bot joins a new guild
//! - When a guild becomes available after a Discord outage
//!
//! The handler performs full synchronization of guild data including:
//! - Guild metadata (name, icon, member count)
//! - All roles in the guild
//! - All text channels in the guild
//! - All guild members (not just app users)
//! - Role assignments for logged-in app users
//!
//! To prevent excessive database load from frequent bot restarts, a 30-minute backoff
//! is enforced. Full synchronization only occurs if 30+ minutes have passed since the
//! last sync. Guild metadata (name, icon) is always updated regardless of the backoff.

use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{Context, Guild};

use crate::server::data::discord::DiscordGuildRepository;
use crate::server::service::discord::{
    DiscordGuildChannelService, DiscordGuildMemberService, DiscordGuildRoleService,
    UserDiscordGuildRoleService,
};

/// Maximum number of members to fetch per API request.
///
/// Discord's API supports up to 1000 members per request. Using the maximum
/// reduces the number of API calls needed for large guilds.
static MEMBERS_PER_REQUEST: u64 = 1000;

/// Minimum time between full guild synchronizations in minutes.
///
/// Full guild syncs (roles, channels, all members) are expensive operations.
/// This backoff prevents excessive syncs when the bot restarts frequently,
/// while ensuring the data stays reasonably fresh.
static SYNC_BACKOFF_MINUTES: i64 = 30;

/// Handles the guild_create event when a guild becomes available or the bot joins a new guild.
///
/// This event fires in multiple scenarios:
/// - On bot startup for each guild the bot is already in
/// - When the bot joins a new guild
/// - When a guild becomes available after an outage
///
/// The handler always updates basic guild metadata (name, icon, member count), then
/// checks if a full synchronization is needed. Full sync occurs only if 30+ minutes
/// have passed since the last sync, preventing excessive database load from frequent
/// bot restarts while keeping data reasonably current.
///
/// Full synchronization includes:
/// 1. Guild roles - All roles with their names, colors, and permissions
/// 2. Guild channels - All text channels available for notifications
/// 3. Guild members - All members with usernames and nicknames (requires GUILD_MEMBERS intent)
/// 4. User roles - Role assignments for logged-in app users only
///
/// Member fetching uses pagination to handle guilds of any size, fetching up to 1000
/// members per API request until all members have been retrieved.
///
/// # Arguments
/// - `db` - Database connection for storing guild data
/// - `ctx` - Discord context for making API requests (used for member pagination)
/// - `guild` - Guild data from Discord including roles, channels, and partial member list
/// - `_is_new` - Whether this is a new guild join (unused, required by event handler signature)
pub async fn handle_guild_create(
    db: &DatabaseConnection,
    ctx: Context,
    guild: Guild,
    _is_new: Option<bool>,
) {
    let guild_id = guild.id.get();
    let guild_name = guild.name.clone();
    let guild_roles = guild.roles.clone();
    let guild_channels = guild.channels.clone();

    tracing::debug!(
        "Guild create event: {} ({}) - member_count: {}",
        guild_name,
        guild_id,
        guild.member_count,
    );

    let guild_repo = DiscordGuildRepository::new(db);

    // Always upsert basic guild metadata (name, icon, member count)
    if let Err(e) = guild_repo.upsert(guild).await {
        tracing::error!(
            "Failed to upsert guild {} ({}): {:?}",
            guild_id,
            guild_name,
            e
        );
        return;
    }

    // Check if a full sync is needed (30-minute backoff)
    let needs_sync = match guild_repo.needs_sync(guild_id).await {
        Ok(needs) => needs,
        Err(e) => {
            tracing::error!("Failed to check if guild {} needs sync: {:?}", guild_id, e);
            return;
        }
    };

    if !needs_sync {
        tracing::debug!(
            "Skipping full sync for guild {} (synced within last {} minutes)",
            guild_id,
            SYNC_BACKOFF_MINUTES
        );
        return;
    }

    tracing::trace!(
        "Performing full sync for guild {} ({})",
        guild_id,
        guild_name
    );

    // Sync all roles in the guild
    let role_service = DiscordGuildRoleService::new(db);

    if let Err(e) = role_service.update_roles(guild_id, &guild_roles).await {
        tracing::error!("Failed to update guild {} roles: {:?}", guild_id, e);
    } else {
        tracing::debug!("Updated {} roles for guild {}", guild_roles.len(), guild_id);
    }

    // Sync all text channels in the guild
    let channel_service = DiscordGuildChannelService::new(db);

    if let Err(e) = channel_service
        .update_channels(guild_id, &guild_channels)
        .await
    {
        tracing::error!("Failed to update guild {} channels: {:?}", guild_id, e);
    } else {
        tracing::debug!(
            "Updated {} channels for guild {}",
            guild_channels.len(),
            guild_id
        );
    }

    // Fetch ALL members from Discord API with pagination
    // This requires the GUILD_MEMBERS privileged intent
    let mut all_members = Vec::new();
    let mut after: Option<u64> = None;

    loop {
        match ctx
            .http
            .get_guild_members(guild_id.into(), Some(MEMBERS_PER_REQUEST), after)
            .await
        {
            Ok(members) => {
                if members.is_empty() {
                    break;
                }

                tracing::debug!(
                    "Fetched {} members from Discord API for guild {} (total so far: {})",
                    members.len(),
                    guild_id,
                    all_members.len() + members.len()
                );

                // Set up pagination for next iteration
                after = members.last().map(|m| m.user.id.get());

                let fetched_count = members.len();

                // Add to our collection
                all_members.extend(members);

                // If we got less than the maximum, we've reached the end
                if fetched_count < MEMBERS_PER_REQUEST as usize {
                    break;
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to fetch guild {} members from API: {:?}",
                    guild_id,
                    e
                );
                break;
            }
        }
    }

    tracing::trace!(
        "Fetched total of {} members for guild {}",
        all_members.len(),
        guild_id
    );

    // Convert to the format needed for sync: (user_id, username, nickname)
    let member_data: Vec<(u64, String, Option<String>)> = all_members
        .iter()
        .map(|m| (m.user.id.get(), m.user.name.clone(), m.nick.clone()))
        .collect();

    // Sync ALL guild members (not just logged-in users)
    // This enables @mentions of any guild member in fleet notifications
    let member_service = DiscordGuildMemberService::new(db);
    if let Err(e) = member_service
        .sync_guild_members(guild_id, &member_data)
        .await
    {
        tracing::error!("Failed to sync guild {} members: {:?}", guild_id, e);
    } else {
        tracing::debug!(
            "Synced {} members for guild {}",
            member_data.len(),
            guild_id
        );
    }

    // Sync role memberships for logged-in users only
    // Only users with app accounts need role assignments for permission checks
    let user_role_service = UserDiscordGuildRoleService::new(db);
    if let Err(e) = user_role_service
        .sync_guild_member_roles(guild_id, &all_members)
        .await
    {
        tracing::error!("Failed to sync guild {} member roles: {:?}", guild_id, e);
    } else {
        tracing::debug!("Synced member roles for guild {}", guild_id);
    }

    // Update last sync timestamp after successful sync
    if let Err(e) = guild_repo.update_last_sync(guild_id).await {
        tracing::error!(
            "Failed to update guild {} last sync timestamp: {:?}",
            guild_id,
            e
        );
    } else {
        tracing::debug!("Successfully completed full sync for guild {}", guild_id);
    }
}
