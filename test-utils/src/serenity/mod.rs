//! Test factories for creating Serenity API objects.
//!
//! This module provides factory functions for creating mock Serenity structs
//! (Guild, Role, Channel, etc.) for testing purposes. These factories create
//! valid Serenity objects by deserializing JSON, simulating what Discord's API
//! would return.
//!
//! # Overview
//!
//! When testing code that interacts with Discord's API via Serenity, you often
//! need to create mock Serenity structs. These factories provide a consistent
//! way to create these objects with sensible defaults while allowing customization
//! of key fields.
//!
//! # Usage
//!
//! ```rust,ignore
//! use test_utils::serenity::{guild::create_test_guild, role::create_test_role};
//!
//! #[tokio::test]
//! async fn test_guild_sync() {
//!     // Create a test guild
//!     let guild = create_test_guild(123456789, "Test Guild", Some("abc123"));
//!
//!     // Create test roles
//!     let admin_role = create_test_role(111111111, "Admin", 0xFF0000, 10);
//!     let member_role = create_test_role(222222222, "Member", 0x00FF00, 1);
//!
//!     // Use in your tests...
//! }
//! ```
//!
//! # Available Factories
//!
//! - `guild::create_test_guild` - Create Serenity Guild objects
//! - `role::create_test_role` - Create Serenity Role objects

pub mod guild;
pub mod role;

// Re-export commonly used functions for convenience
pub use guild::create_test_guild;
pub use role::create_test_role;
