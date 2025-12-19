//! Ping format factory for creating test ping format entities.
//!
//! This module provides factory methods for creating ping format entities with
//! sensible defaults, reducing boilerplate in tests. The factory supports
//! customization through a builder pattern.

use crate::factory::helpers::next_id;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for creating test ping formats with customizable fields.
///
/// Provides a builder pattern for creating ping format entities with default
/// values that can be overridden as needed for specific test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::factory::ping_format::PingFormatFactory;
///
/// let ping_format = PingFormatFactory::new(&db, "guild_123")
///     .name("Custom Format")
///     .build()
///     .await?;
/// ```
pub struct PingFormatFactory<'a> {
    db: &'a DatabaseConnection,
    guild_id: String,
    name: String,
}

impl<'a> PingFormatFactory<'a> {
    /// Creates a new PingFormatFactory with default values.
    ///
    /// Defaults:
    /// - name: `"Ping Format {id}"` where id is auto-incremented
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    /// - `guild_id` - Discord guild ID this format belongs to
    ///
    /// # Returns
    /// - `PingFormatFactory` - New factory instance with defaults
    pub fn new(db: &'a DatabaseConnection, guild_id: impl Into<String>) -> Self {
        let id = next_id();
        Self {
            db,
            guild_id: guild_id.into(),
            name: format!("Ping Format {}", id),
        }
    }

    /// Sets the ping format name.
    ///
    /// # Arguments
    /// - `name` - Display name for the ping format
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Builds and inserts the ping format entity into the database.
    ///
    /// # Returns
    /// - `Ok(entity::ping_format::Model)` - Created ping format entity
    /// - `Err(DbErr)` - Database error during insert
    pub async fn build(self) -> Result<entity::ping_format::Model, DbErr> {
        entity::ping_format::ActiveModel {
            id: ActiveValue::NotSet,
            guild_id: ActiveValue::Set(self.guild_id),
            name: ActiveValue::Set(self.name),
        }
        .insert(self.db)
        .await
    }
}

/// Creates a ping format with default values for the specified guild.
///
/// Shorthand for `PingFormatFactory::new(db, guild_id).build().await`.
///
/// # Arguments
/// - `db` - Database connection
/// - `guild_id` - Discord guild ID
///
/// # Returns
/// - `Ok(entity::ping_format::Model)` - Created ping format entity
/// - `Err(DbErr)` - Database error during insert
///
/// # Example
///
/// ```rust,ignore
/// let ping_format = create_ping_format(&db, "guild_123").await?;
/// ```
pub async fn create_ping_format(
    db: &DatabaseConnection,
    guild_id: impl Into<String>,
) -> Result<entity::ping_format::Model, DbErr> {
    PingFormatFactory::new(db, guild_id).build().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::TestBuilder;
    use crate::factory::discord_guild::create_guild;
    use entity::prelude::*;

    #[tokio::test]
    async fn creates_ping_format_with_defaults() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;

        assert_eq!(ping_format.guild_id, guild.guild_id);
        assert!(!ping_format.name.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn creates_ping_format_with_custom_values() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = PingFormatFactory::new(db, &guild.guild_id)
            .name("Custom Format")
            .build()
            .await?;

        assert_eq!(ping_format.guild_id, guild.guild_id);
        assert_eq!(ping_format.name, "Custom Format");

        Ok(())
    }

    #[tokio::test]
    async fn creates_multiple_unique_ping_formats() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let format1 = create_ping_format(db, &guild.guild_id).await?;
        let format2 = create_ping_format(db, &guild.guild_id).await?;

        assert_ne!(format1.id, format2.id);
        assert_ne!(format1.name, format2.name);
        assert_eq!(format1.guild_id, format2.guild_id);

        Ok(())
    }
}
