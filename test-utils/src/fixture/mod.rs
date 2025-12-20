//! Test fixtures providing reusable test data without database insertion.
//!
//! This module contains fixture functions that create in-memory test data structures
//! for use in unit tests and as default values for factories. Unlike factories,
//! fixtures do NOT insert data into the database.
//!
//! # When to Use Fixtures
//!
//! - **Unit testing**: Test business logic without database overhead
//! - **Mocking**: Create test data for mocking repository responses
//! - **Default values**: Provide consistent defaults for factory builders
//! - **Serialization tests**: Test DTO conversion without persistence
//!
//! # Example
//!
//! ```rust,ignore
//! use test_utils::fixture;
//!
//! // Create in-memory entity model (no DB)
//! let user = fixture::user::entity();
//!
//! // Create with custom fields
//! let admin = fixture::user::entity_builder()
//!     .admin(true)
//!     .build();
//! ```

pub mod discord_guild;
pub mod discord_guild_channel;
pub mod discord_guild_role;
pub mod fleet;
pub mod fleet_category;
pub mod ping_format;
pub mod ping_format_field;
pub mod user;
pub mod user_discord_guild_role;

pub use discord_guild::{
    entity as discord_guild_entity, entity_builder as discord_guild_entity_builder,
};
pub use discord_guild_channel::{
    entity as discord_guild_channel_entity, entity_builder as discord_guild_channel_entity_builder,
};
pub use discord_guild_role::{
    entity as discord_guild_role_entity, entity_builder as discord_guild_role_entity_builder,
};
pub use fleet::{entity as fleet_entity, entity_builder as fleet_entity_builder};
pub use fleet_category::{
    entity as fleet_category_entity, entity_builder as fleet_category_entity_builder,
};
pub use ping_format::{entity as ping_format_entity, entity_builder as ping_format_entity_builder};
pub use ping_format_field::{
    entity as ping_format_field_entity, entity_builder as ping_format_field_entity_builder,
};
pub use user::{entity as user_entity, entity_builder as user_entity_builder};
pub use user_discord_guild_role::{
    entity as user_discord_guild_role_entity,
    entity_builder as user_discord_guild_role_entity_builder,
};
