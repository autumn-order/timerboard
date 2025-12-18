//! Discord domain models and data structures.
//!
//! Provides domain models for Discord entities including guilds, channels, roles, and members.
//! These models are used throughout the service layer and provide type-safe representations
//! with conversion methods for entity and DTO transformations.

pub mod channel;
pub mod guild;
pub mod guild_member;
pub mod role;
pub mod user_guild_role;

pub use channel::DiscordGuildChannel;
pub use guild::DiscordGuild;
pub use guild_member::DiscordGuildMember;
pub use role::DiscordGuildRole;
pub use user_guild_role::UserDiscordGuildRole;
