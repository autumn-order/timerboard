//! Role event handlers for Discord guild roles.
//!
//! This module handles Discord events related to guild roles, keeping the database
//! synchronized with Discord's role state. Roles are tracked to enable:
//! - Role-based access control for fleet management
//! - Role mentions in fleet notifications
//! - Permission checking for administrative actions
//!
//! All roles in guilds are tracked, not just roles assigned to application users.
//! This allows the application to display accurate role information in the UI and
//! support role-based features even for users who haven't logged into the app yet.

use dioxus_logger::tracing;
use sea_orm::DatabaseConnection;
use serenity::all::{Context, GuildId, Role, RoleId};

use crate::server::data::discord::DiscordGuildRoleRepository;

/// Handles the guild_role_create event when a role is created in a guild.
///
/// Adds the role to the database, making it available for:
/// - Role-based access control configuration
/// - Role mentions in fleet notifications
/// - Permission checks
///
/// # Arguments
/// - `db` - Database connection for creating the role record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `new` - The newly created role from Discord
pub async fn handle_guild_role_create(db: &DatabaseConnection, _ctx: Context, new: Role) {
    let guild_id = new.guild_id.get();
    let role_repo = DiscordGuildRoleRepository::new(db);

    if let Err(e) = role_repo.upsert(guild_id, &new).await {
        tracing::error!(
            "Failed to upsert new role {} in guild {}: {:?}",
            new.name,
            guild_id,
            e
        );
    } else {
        tracing::debug!("Created role {} in guild {}", new.name, guild_id);
    }
}

/// Handles the guild_role_update event when a role is updated in a guild.
///
/// Updates the role's information (name, color, permissions, position, etc.) in
/// the database. This ensures the UI displays current role information and that
/// permission checks use up-to-date role data.
///
/// # Arguments
/// - `db` - Database connection for updating the role record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `_old` - Previous role state if available (unused)
/// - `new` - Updated role state from Discord
pub async fn handle_guild_role_update(
    db: &DatabaseConnection,
    _ctx: Context,
    _old: Option<Role>,
    new: Role,
) {
    let guild_id = new.guild_id.get();
    let role_repo = DiscordGuildRoleRepository::new(db);

    if let Err(e) = role_repo.upsert(guild_id, &new).await {
        tracing::error!(
            "Failed to upsert updated role {} in guild {}: {:?}",
            new.name,
            guild_id,
            e
        );
    } else {
        tracing::debug!("Updated role {} in guild {}", new.name, guild_id);
    }
}

/// Handles the guild_role_delete event when a role is deleted from a guild.
///
/// Removes the role from the database. User role assignments that reference this
/// role are automatically cleaned up via database CASCADE constraints, preventing
/// orphaned role assignment records.
///
/// # Arguments
/// - `db` - Database connection for deleting the role record
/// - `_ctx` - Discord context (unused, required by event handler signature)
/// - `guild_id` - ID of the guild the role was deleted from
/// - `removed_role_id` - ID of the deleted role
/// - `_removed_role_data_if_in_cache` - Role data if it was in cache (unused)
pub async fn handle_guild_role_delete(
    db: &DatabaseConnection,
    _ctx: Context,
    guild_id: GuildId,
    removed_role_id: RoleId,
    _removed_role_data_if_in_cache: Option<Role>,
) {
    let role_repo = DiscordGuildRoleRepository::new(db);

    if let Err(e) = role_repo.delete(removed_role_id.get()).await {
        tracing::error!(
            "Failed to delete role {} from guild {}: {:?}",
            removed_role_id,
            guild_id,
            e
        );
    } else {
        tracing::info!("Deleted role {} from guild {}", removed_role_id, guild_id);
    }
}
