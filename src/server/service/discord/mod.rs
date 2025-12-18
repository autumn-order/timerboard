//! Discord API integration services.
//!
//! This module provides service abstractions for interacting with the Discord API through
//! the Serenity library. These services handle Discord-specific operations such as fetching
//! guild information, managing channels and roles, and synchronizing user access permissions.

pub mod channel;
pub mod guild;
pub mod guild_member;
pub mod role;
pub mod user_guild_role;

pub use channel::DiscordGuildChannelService;
pub use guild::DiscordGuildService;
pub use guild_member::DiscordGuildMemberService;
pub use role::DiscordGuildRoleService;
pub use user_guild_role::UserDiscordGuildRoleService;
