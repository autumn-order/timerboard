//! Discord event handlers for bot integration.
//!
//! This module contains event handlers for processing Discord gateway events received
//! by the bot. Each handler module processes specific types of events and updates the
//! application database to maintain synchronization with Discord's state.
//!
//! The handlers are invoked automatically by the Serenity framework when events are
//! received from Discord's gateway. The `Handler` struct implements Serenity's
//! `EventHandler` trait and delegates to the appropriate handler functions.
//!
//! # Event Categories
//!
//! - **Ready** (`ready`) - Bot connection and initialization events
//! - **Guild** (`guild`) - Guild availability, joins, and full synchronization
//! - **Role** (`role`) - Role creation, updates, and deletion within guilds
//! - **Channel** (`channel`) - Channel creation, updates, and deletion within guilds
//! - **Member** (`member`) - Member joins, leaves, and updates (roles, nicknames)
//! - **Message** (`message`) - Message creation for tracking fleet list visibility
//!
//! # Synchronization Strategy
//!
//! The bot maintains several types of Discord data in the database:
//!
//! - **Guild metadata** - Name, icon, and last sync timestamp
//! - **Roles** - All roles in guilds with their names and permissions
//! - **Channels** - Text channels for notification configuration
//! - **Members** - All guild members (not just app users) for mention support
//! - **User roles** - Role assignments for logged-in app users only
//!
//! Full guild synchronization (roles, channels, all members) occurs:
//! - When the bot first joins a guild
//! - On bot startup if 30+ minutes have passed since last sync
//!
//! Individual events (role updates, member joins, etc.) trigger incremental updates
//! to keep the database in sync without requiring full re-synchronization.

use sea_orm::DatabaseConnection;
use serenity::all::{
    Context, EventHandler, Guild, GuildChannel, GuildId, GuildMemberUpdateEvent, Member, Message,
    Ready, Role, RoleId, User,
};
use serenity::async_trait;

pub mod channel;
pub mod guild;
pub mod member;
pub mod message;
pub mod ready;
pub mod role;

/// Discord bot event handler with database access.
///
/// Implements Serenity's `EventHandler` trait to process Discord gateway events.
/// Each event handler method delegates to the appropriate handler function in
/// the respective module, passing the database connection for state updates.
///
/// The handler is configured when the bot client is initialized and processes
/// events asynchronously as they are received from Discord's gateway.
pub struct Handler {
    /// Database connection for updating application state based on Discord events.
    pub db: DatabaseConnection,
}

impl Handler {
    /// Creates a new event handler with database access.
    ///
    /// # Arguments
    /// - `db` - Database connection for the handler to use when processing events
    ///
    /// # Returns
    /// - `Handler` - New event handler instance ready to process Discord events
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventHandler for Handler {
    /// Called when the bot is ready and connected to Discord.
    ///
    /// This event fires after successful authentication and initial gateway handshake.
    async fn ready(&self, ctx: Context, ready: Ready) {
        ready::handle_ready(ctx, ready).await;
    }

    /// Called when a guild becomes available or the bot joins a new guild.
    ///
    /// This event fires:
    /// - On bot startup for each guild the bot is in
    /// - When the bot joins a new guild
    /// - When a guild becomes available after an outage
    ///
    /// Performs full synchronization of guild data (roles, channels, members) with
    /// a 30-minute backoff to prevent excessive syncs on frequent bot restarts.
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        guild::handle_guild_create(&self.db, ctx, guild, is_new).await;
    }

    /// Called when a role is created in a guild.
    ///
    /// Creates or updates the role in the database to keep role information
    /// synchronized for permission checks and role mentions.
    async fn guild_role_create(&self, ctx: Context, new: Role) {
        role::handle_guild_role_create(&self.db, ctx, new).await;
    }

    /// Called when a role is updated in a guild.
    ///
    /// Updates the role's information (name, color, permissions, etc.) in the
    /// database to maintain accurate role data.
    async fn guild_role_update(&self, ctx: Context, old: Option<Role>, new: Role) {
        role::handle_guild_role_update(&self.db, ctx, old, new).await;
    }

    /// Called when a role is deleted from a guild.
    ///
    /// Removes the role from the database. User role assignments are automatically
    /// cleaned up via database CASCADE constraints.
    async fn guild_role_delete(
        &self,
        ctx: Context,
        guild_id: GuildId,
        removed_role_id: RoleId,
        removed_role_data_if_in_cache: Option<Role>,
    ) {
        role::handle_guild_role_delete(
            &self.db,
            ctx,
            guild_id,
            removed_role_id,
            removed_role_data_if_in_cache,
        )
        .await;
    }

    /// Called when a member joins a guild.
    ///
    /// Adds the member to the database (tracked for all members, not just app users).
    /// If the user has an application account, also syncs their role assignments.
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        member::handle_guild_member_addition(&self.db, ctx, new_member).await;
    }

    /// Called when a member leaves a guild.
    ///
    /// Removes the member from the database. Role assignments for app users are
    /// automatically cleaned up via database CASCADE constraints.
    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        member::handle_guild_member_removal(
            &self.db,
            ctx,
            guild_id,
            user,
            member_data_if_available,
        )
        .await;
    }

    /// Called when a member is updated in a guild.
    ///
    /// Updates member information (username, nickname) and syncs role assignments
    /// for app users when roles change.
    async fn guild_member_update(
        &self,
        ctx: Context,
        old: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        member::handle_guild_member_update(&self.db, ctx, old, new, event).await;
    }

    /// Called when a channel is created in a guild.
    ///
    /// Adds text channels to the database for use in notification configuration.
    /// Non-text channels are ignored.
    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        channel::handle_channel_create(&self.db, ctx, channel).await;
    }

    /// Called when a channel is updated in a guild.
    ///
    /// Updates text channel information (name, permissions, etc.) in the database.
    /// Non-text channels are ignored.
    async fn channel_update(&self, ctx: Context, old: Option<GuildChannel>, new: GuildChannel) {
        channel::handle_channel_update(&self.db, ctx, old, new).await;
    }

    /// Called when a channel is deleted from a guild.
    ///
    /// Removes the channel from the database. Associated notification configurations
    /// are automatically cleaned up via database CASCADE constraints.
    async fn channel_delete(
        &self,
        ctx: Context,
        channel: GuildChannel,
        messages: Option<Vec<Message>>,
    ) {
        channel::handle_channel_delete(&self.db, ctx, channel, messages).await;
    }

    /// Called when a message is sent in a channel.
    ///
    /// Tracks message timestamps in channels with fleet list messages to determine
    /// when the list has been buried by other messages and needs to be reposted.
    async fn message(&self, ctx: Context, message: Message) {
        message::handle_message(&self.db, ctx, message).await;
    }
}
