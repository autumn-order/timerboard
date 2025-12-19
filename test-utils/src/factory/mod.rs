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
//! use test_utils::factory::user::UserFactory;
//!
//! let user = UserFactory::new(&db)
//!     .discord_id("987654321")
//!     .name("CustomUser")
//!     .admin(true)
//!     .build()
//!     .await?;
//! ```
//!
//! # Available Factories
//!
//! - `user` - Create user entities
//! - `discord_guild` - Create Discord guild entities
//! - `ping_format` - Create ping format entities
//! - `fleet_category` - Create fleet category entities
//! - `fleet` - Create fleet entities
//! - `helpers` - Convenience methods for creating entities with dependencies

pub mod discord_guild;
pub mod fleet;
pub mod fleet_category;
pub mod helpers;
pub mod ping_format;
pub mod user;
