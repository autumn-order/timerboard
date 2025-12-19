//! Discord guild factory for creating test guild entities.
//!
//! This module provides factory methods for creating Discord guild entities with
//! sensible defaults, reducing boilerplate in tests. The factory supports
//! customization through a builder pattern.

use crate::factory::helpers::next_id;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for creating test Discord guilds with customizable fields.
///
/// Provides a builder pattern for creating Discord guild entities with default
/// values that can be overridden as needed for specific test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::factory::discord_guild::DiscordGuildFactory;
///
/// let guild = DiscordGuildFactory::new(&db)
///     .guild_id("987654321")
///     .name("CustomGuild")
///     .build()
///     .await?;
/// ```
pub struct DiscordGuildFactory<'a> {
    db: &'a DatabaseConnection,
    guild_id: String,
    name: String,
    icon_hash: Option<String>,
}

impl<'a> DiscordGuildFactory<'a> {
    /// Creates a new DiscordGuildFactory with default values.
    ///
    /// Defaults:
    /// - guild_id: `"guild_{id}"` where id is auto-incremented
    /// - name: `"Guild {id}"`
    /// - icon_hash: `None`
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    ///
    /// # Returns
    /// - `DiscordGuildFactory` - New factory instance with defaults
    pub fn new(db: &'a DatabaseConnection) -> Self {
        let id = next_id();
        Self {
            db,
            guild_id: id.to_string(),
            name: format!("Guild {}", id),
            icon_hash: None,
        }
    }

    /// Sets the guild ID.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID as string
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn guild_id(mut self, guild_id: impl Into<String>) -> Self {
        self.guild_id = guild_id.into();
        self
    }

    /// Sets the guild name.
    ///
    /// # Arguments
    /// - `name` - Display name for the guild
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the icon hash for the guild.
    ///
    /// # Arguments
    /// - `icon_hash` - Optional Discord icon hash
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn icon_hash(mut self, icon_hash: Option<String>) -> Self {
        self.icon_hash = icon_hash;
        self
    }

    /// Builds and inserts the guild entity into the database.
    ///
    /// # Returns
    /// - `Ok(entity::discord_guild::Model)` - Created guild entity
    /// - `Err(DbErr)` - Database error during insert
    pub async fn build(self) -> Result<entity::discord_guild::Model, DbErr> {
        entity::discord_guild::ActiveModel {
            guild_id: ActiveValue::Set(self.guild_id),
            name: ActiveValue::Set(self.name),
            icon_hash: ActiveValue::Set(self.icon_hash),
            last_sync_at: ActiveValue::Set(Utc::now()),
        }
        .insert(self.db)
        .await
    }
}

/// Creates a Discord guild with default values.
///
/// Shorthand for `DiscordGuildFactory::new(db).build().await`.
///
/// # Arguments
/// - `db` - Database connection
///
/// # Returns
/// - `Ok(entity::discord_guild::Model)` - Created guild entity
/// - `Err(DbErr)` - Database error during insert
///
/// # Example
///
/// ```rust,ignore
/// let guild = create_guild(&db).await?;
/// ```
pub async fn create_guild(db: &DatabaseConnection) -> Result<entity::discord_guild::Model, DbErr> {
    DiscordGuildFactory::new(db).build().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::TestBuilder;
    use entity::prelude::*;

    #[tokio::test]
    async fn creates_guild_with_defaults() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;

        assert!(!guild.guild_id.is_empty());
        assert!(!guild.name.is_empty());
        assert!(guild.icon_hash.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn creates_guild_with_custom_values() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = DiscordGuildFactory::new(db)
            .guild_id("987654321")
            .name("CustomGuild")
            .icon_hash(Some("abcd1234".to_string()))
            .build()
            .await?;

        assert_eq!(guild.guild_id, "987654321");
        assert_eq!(guild.name, "CustomGuild");
        assert_eq!(guild.icon_hash, Some("abcd1234".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn creates_multiple_unique_guilds() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild1 = create_guild(db).await?;
        let guild2 = create_guild(db).await?;

        assert_ne!(guild1.guild_id, guild2.guild_id);
        assert_ne!(guild1.name, guild2.name);

        Ok(())
    }
}
