//! Factory methods for creating test data.
//!
//! This module provides factory methods for creating test entities with sensible defaults,
//! reducing boilerplate in tests. Factories automatically handle dependencies and foreign
//! key relationships, making tests more concise and maintainable.
//!
//! # Overview
//!
//! Each entity has its own factory module with both a `Factory` struct for customization
//! and a `create_*` convenience function for quick default creation.
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use test_utils::factory;
//!
//! #[tokio::test]
//! async fn test_example() -> Result<(), sea_orm::DbErr> {
//!     let db = /* ... */;
//!
//!     // Create with defaults
//!     let user = factory::user::create_user(&db).await?;
//!     let guild = factory::discord_guild::create_guild(&db).await?;
//!
//!     // Create with all dependencies
//!     let (user, guild, ping_format, category, fleet) =
//!         factory::helpers::create_fleet_with_dependencies(&db).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Customization
//!
//! Use the factory builders for custom values:
//!
//! ```rust,ignore
//! use test_utils::factory;
//!
//! // Using builder pattern for customization
//! let user = factory::user::UserFactory::new(&db)
//!     .discord_id("987654321")
//!     .name("CustomUser")
//!     .admin(true)
//!     .build()
//!     .await?;
//!
//! // Using convenience functions with custom values
//! let role = factory::create_guild_role_with_position(&db, &guild.guild_id, "123", 10).await?;
//! let channel = factory::create_guild_channel_with_position(&db, &guild.guild_id, "456", 1).await?;
//! ```
//!
//! # Available Factories
//!
//! - `user` - Create user entities
//! - `discord_guild` - Create Discord guild entities
//! - `discord_guild_role` - Create Discord guild role entities
//! - `discord_guild_channel` - Create Discord guild channel entities
//! - `ping_format` - Create ping format entities
//! - `fleet_category` - Create fleet category entities
//! - `fleet` - Create fleet entities
//! - `user_discord_guild_role` - Create user-guild-role relationship entities
//! - `helpers` - Convenience methods for creating entities with dependencies

pub mod discord_guild;
pub mod discord_guild_channel;
pub mod discord_guild_role;
pub mod fleet;
pub mod fleet_category;
pub mod helpers;
pub mod ping_format;
pub mod ping_format_field;
pub mod user;
pub mod user_discord_guild_role;

// Re-export commonly used factory functions for concise usage
pub use discord_guild::create_guild;
pub use discord_guild_channel::{create_guild_channel, create_guild_channel_with_position};
pub use discord_guild_role::{create_guild_role, create_guild_role_with_position};
pub use fleet::create_fleet;
pub use fleet_category::create_category;
pub use ping_format::create_ping_format;
pub use user::create_user;
pub use user_discord_guild_role::{create_user_guild_role, create_user_guild_roles};
