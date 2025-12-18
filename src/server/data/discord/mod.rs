//! Discord data repositories for database operations.
//!
//! This module provides database repositories for Discord-related entities including guilds,
//! channels, roles, members, and user-guild-role relationships. These repositories handle
//! CRUD operations and queries for Discord data synchronized from the Discord API.

pub mod channel;
pub mod guild;
pub mod guild_member;
pub mod role;
pub mod user_guild_role;

pub use channel::DiscordGuildChannelRepository;
pub use guild::DiscordGuildRepository;
pub use guild_member::DiscordGuildMemberRepository;
pub use role::DiscordGuildRoleRepository;
pub use user_guild_role::UserDiscordGuildRoleRepository;
