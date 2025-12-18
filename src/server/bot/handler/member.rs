//! Member event handlers for Discord guild members.
//!
//! This module handles Discord events related to guild members (users in a guild),
//! keeping the database synchronized with Discord's member state. The handlers track:
//! - All guild members (not just app users) for mention support in notifications
//! - Member usernames and nicknames for display purposes
//! - Role assignments for logged-in app users only
//!
//! Tracking all members (not just app users) enables the application to:
//! - Show accurate member lists in the UI
//! - Support @mentions of any guild member in fleet notifications
//! - Pre-populate member data when users log into the application
//!
//! Role synchronization only occurs for members who have logged into the application,
//! as role-based permissions only apply to authenticated users.

use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{Context, GuildId, GuildMemberUpdateEvent, Member, User};

use crate::server::service::discord::{DiscordGuildMemberService, UserDiscordGuildRoleService};

/// Handles the guild_member_addition event when a member joins a guild.
///
/// Adds the member to the database, tracking their user ID, guild membership,
/// username, and nickname. This ensures all guild members are available for
/// mentions in fleet notifications and for display in the UI.
///
/// If the member has an application account (has logged in), also synchronizes
/// their role assignments for permission checks. Members without app accounts
/// are tracked but do not have role assignments synchronized.
///
/// # Arguments
/// - `db` - Database connection for creating the member record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `new_member` - The member who joined the guild from Discord
pub async fn handle_guild_member_addition(
    db: &DatabaseConnection,
    _ctx: Context,
    new_member: Member,
) {
    let user_id = new_member.user.id.get();
    let guild_id = new_member.guild_id.get();
    let username = new_member.user.name.clone();
    let nickname = new_member.nick.clone();

    tracing::debug!(
        "Member {} ({}) joined guild {}",
        username,
        user_id,
        guild_id
    );

    // Add member to guild_member table (tracks ALL members)
    let member_service = DiscordGuildMemberService::new(db);
    if let Err(e) = member_service
        .upsert_member(user_id, guild_id, username.clone(), nickname.clone())
        .await
    {
        tracing::error!(
            "Failed to add guild member {} ({}) to guild {}: {:?}",
            username,
            user_id,
            guild_id,
            e
        );
        return;
    }

    // If this user has an application account, sync their roles
    let user_role_service = UserDiscordGuildRoleService::new(db);
    if let Err(e) = user_role_service
        .sync_user_roles(user_id, &new_member)
        .await
    {
        // This will fail silently if user doesn't have an app account - that's fine
        tracing::debug!(
            "Did not sync roles for user {} ({}) in guild {} (likely not logged into app): {:?}",
            username,
            user_id,
            guild_id,
            e
        );
    } else {
        tracing::debug!(
            "Synced roles for user {} ({}) in guild {}",
            username,
            user_id,
            guild_id
        );
    }
}

/// Handles the guild_member_removal event when a member leaves a guild.
///
/// Removes the member from the database. If the member has an application account,
/// their role assignments are automatically cleaned up via database CASCADE constraints
/// when the member record is deleted.
///
/// This ensures the database accurately reflects current guild membership and prevents
/// stale member data from accumulating.
///
/// # Arguments
/// - `db` - Database connection for deleting the member record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `guild_id` - ID of the guild the member left
/// - `user` - The user who left the guild
/// - `_member_data_if_available` - Member data if it was in cache (unused)
pub async fn handle_guild_member_removal(
    db: &DatabaseConnection,
    _ctx: Context,
    guild_id: GuildId,
    user: User,
    _member_data_if_available: Option<Member>,
) {
    let user_id = user.id.get();
    let guild_id = guild_id.get();

    tracing::debug!("Member {} ({}) left guild {}", user.name, user_id, guild_id);

    // Remove member from guild_member table
    let member_service = DiscordGuildMemberService::new(db);
    if let Err(e) = member_service.remove_member(user_id, guild_id).await {
        tracing::error!(
            "Failed to remove guild member {} ({}) from guild {}: {:?}",
            user.name,
            user_id,
            guild_id,
            e
        );
    } else {
        tracing::debug!(
            "Removed member {} ({}) from guild {}",
            user.name,
            user_id,
            guild_id
        );
    }

    // Note: user_discord_guild_role records will be automatically deleted via CASCADE
    // when the guild_member row is deleted (for logged-in users only)
}

/// Handles the guild_member_update event when a member is updated in a guild.
///
/// Updates the member's information (username, nickname) in the database and
/// synchronizes role assignments if the member has an application account.
/// This event fires when:
/// - Member's roles change
/// - Member's nickname changes
/// - Member's avatar changes
/// - Member's timeout status changes
///
/// For logged-in app users, role changes trigger a full role synchronization
/// to ensure permission checks use current role assignments.
///
/// # Arguments
/// - `db` - Database connection for updating the member record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `_old` - Previous member state if available (unused)
/// - `new` - Updated member state from Discord
/// - `_event` - Raw event data (unused)
pub async fn handle_guild_member_update(
    db: &DatabaseConnection,
    _ctx: Context,
    _old: Option<Member>,
    new: Option<Member>,
    _event: GuildMemberUpdateEvent,
) {
    let Some(member) = new else {
        tracing::warn!("Received guild_member_update with no member data");
        return;
    };

    let user_id = member.user.id.get();
    let guild_id = member.guild_id.get();
    let username = member.user.name.clone();
    let nickname = member.nick.clone();

    tracing::debug!(
        "Member {} ({}) updated in guild {}",
        username,
        user_id,
        guild_id
    );

    // Update member in guild_member table (updates username/nickname)
    let member_service = DiscordGuildMemberService::new(db);
    if let Err(e) = member_service
        .upsert_member(user_id, guild_id, username.clone(), nickname.clone())
        .await
    {
        tracing::error!(
            "Failed to update guild member {} ({}) in guild {}: {:?}",
            username,
            user_id,
            guild_id,
            e
        );
        return;
    }

    // If this user has an application account, sync their roles
    let user_role_service = UserDiscordGuildRoleService::new(db);
    if let Err(e) = user_role_service.sync_user_roles(user_id, &member).await {
        tracing::debug!(
            "Did not sync roles for user {} ({}) in guild {} (likely not logged into app): {:?}",
            username,
            user_id,
            guild_id,
            e
        );
    } else {
        tracing::debug!(
            "Synced roles for user {} ({}) in guild {}",
            username,
            user_id,
            guild_id
        );
    }
}
