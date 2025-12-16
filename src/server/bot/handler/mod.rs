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

/// Discord bot event handler
pub struct Handler {
    pub db: DatabaseConnection,
}

impl Handler {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventHandler for Handler {
    /// Called when the bot is ready and connected to Discord
    async fn ready(&self, ctx: Context, ready: Ready) {
        ready::handle_ready(ctx, ready).await;
    }

    /// Called when a guild becomes available or the bot joins a new guild
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        guild::handle_guild_create(&self.db, ctx, guild, is_new).await;
    }

    /// Called when a role is created in a guild
    async fn guild_role_create(&self, ctx: Context, new: Role) {
        role::handle_guild_role_create(&self.db, ctx, new).await;
    }

    /// Called when a role is updated in a guild
    async fn guild_role_update(&self, ctx: Context, old: Option<Role>, new: Role) {
        role::handle_guild_role_update(&self.db, ctx, old, new).await;
    }

    /// Called when a role is deleted from a guild
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

    /// Called when a member joins a guild
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        member::handle_guild_member_addition(&self.db, ctx, new_member).await;
    }

    /// Called when a member leaves a guild
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

    /// Called when a member is updated in a guild (roles, nickname, etc.)
    async fn guild_member_update(
        &self,
        ctx: Context,
        old: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        member::handle_guild_member_update(&self.db, ctx, old, new, event).await;
    }

    /// Called when a channel is created in a guild
    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        channel::handle_channel_create(&self.db, ctx, channel).await;
    }

    /// Called when a channel is updated in a guild
    async fn channel_update(&self, ctx: Context, old: Option<GuildChannel>, new: GuildChannel) {
        channel::handle_channel_update(&self.db, ctx, old, new).await;
    }

    /// Called when a channel is deleted from a guild
    async fn channel_delete(
        &self,
        ctx: Context,
        channel: GuildChannel,
        messages: Option<Vec<Message>>,
    ) {
        channel::handle_channel_delete(&self.db, ctx, channel, messages).await;
    }

    /// Called when a message is sent in a channel
    async fn message(&self, ctx: Context, message: Message) {
        message::handle_message(&self.db, ctx, message).await;
    }
}
