//! Fleet category factory for creating test fleet category entities.
//!
//! This module provides factory methods for creating fleet category entities with
//! sensible defaults, reducing boilerplate in tests. The factory supports
//! customization through a builder pattern.

use crate::factory::helpers::next_id;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for creating test fleet categories with customizable fields.
///
/// Provides a builder pattern for creating fleet category entities with default
/// values that can be overridden as needed for specific test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::factory::fleet_category::FleetCategoryFactory;
///
/// let category = FleetCategoryFactory::new(&db, "guild_123", 1)
///     .name("Custom Category")
///     .ping_cooldown(Some(60))
///     .build()
///     .await?;
/// ```
pub struct FleetCategoryFactory<'a> {
    db: &'a DatabaseConnection,
    guild_id: String,
    ping_format_id: i32,
    name: String,
    ping_cooldown: Option<i32>,
    ping_reminder: Option<i32>,
    max_pre_ping: Option<i32>,
}

impl<'a> FleetCategoryFactory<'a> {
    /// Creates a new FleetCategoryFactory with default values.
    ///
    /// Defaults:
    /// - name: `"Category {id}"` where id is auto-incremented
    /// - ping_cooldown: `None`
    /// - ping_reminder: `None`
    /// - max_pre_ping: `None`
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    /// - `guild_id` - Discord guild ID this category belongs to
    /// - `ping_format_id` - Ping format ID for this category
    ///
    /// # Returns
    /// - `FleetCategoryFactory` - New factory instance with defaults
    pub fn new(
        db: &'a DatabaseConnection,
        guild_id: impl Into<String>,
        ping_format_id: i32,
    ) -> Self {
        let id = next_id();
        Self {
            db,
            guild_id: guild_id.into(),
            ping_format_id,
            name: format!("Category {}", id),
            ping_cooldown: None,
            ping_reminder: None,
            max_pre_ping: None,
        }
    }

    /// Sets the category name.
    ///
    /// # Arguments
    /// - `name` - Display name for the category
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the ping cooldown in minutes.
    ///
    /// # Arguments
    /// - `cooldown` - Cooldown period between pings
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn ping_cooldown(mut self, cooldown: Option<i32>) -> Self {
        self.ping_cooldown = cooldown;
        self
    }

    /// Sets the ping reminder time in minutes.
    ///
    /// # Arguments
    /// - `reminder` - Minutes before fleet time to send reminder
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn ping_reminder(mut self, reminder: Option<i32>) -> Self {
        self.ping_reminder = reminder;
        self
    }

    /// Sets the maximum pre-ping time in minutes.
    ///
    /// # Arguments
    /// - `max_pre_ping` - Maximum minutes before fleet time to allow pings
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn max_pre_ping(mut self, max_pre_ping: Option<i32>) -> Self {
        self.max_pre_ping = max_pre_ping;
        self
    }

    /// Builds and inserts the fleet category entity into the database.
    ///
    /// # Returns
    /// - `Ok(entity::fleet_category::Model)` - Created category entity
    /// - `Err(DbErr)` - Database error during insert
    pub async fn build(self) -> Result<entity::fleet_category::Model, DbErr> {
        entity::fleet_category::ActiveModel {
            id: ActiveValue::NotSet,
            guild_id: ActiveValue::Set(self.guild_id),
            ping_format_id: ActiveValue::Set(self.ping_format_id),
            name: ActiveValue::Set(self.name),
            ping_cooldown: ActiveValue::Set(self.ping_cooldown),
            ping_reminder: ActiveValue::Set(self.ping_reminder),
            max_pre_ping: ActiveValue::Set(self.max_pre_ping),
        }
        .insert(self.db)
        .await
    }
}

/// Creates a fleet category with default values for the specified guild and ping format.
///
/// Shorthand for `FleetCategoryFactory::new(db, guild_id, ping_format_id).build().await`.
///
/// # Arguments
/// - `db` - Database connection
/// - `guild_id` - Discord guild ID
/// - `ping_format_id` - Ping format ID
///
/// # Returns
/// - `Ok(entity::fleet_category::Model)` - Created category entity
/// - `Err(DbErr)` - Database error during insert
///
/// # Example
///
/// ```rust,ignore
/// let category = create_category(&db, "guild_123", 1).await?;
/// ```
pub async fn create_category(
    db: &DatabaseConnection,
    guild_id: impl Into<String>,
    ping_format_id: i32,
) -> Result<entity::fleet_category::Model, DbErr> {
    FleetCategoryFactory::new(db, guild_id, ping_format_id)
        .build()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::TestBuilder;
    use crate::factory::discord_guild::create_guild;
    use crate::factory::ping_format::create_ping_format;
    use entity::prelude::*;

    #[tokio::test]
    async fn creates_category_with_defaults() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(FleetCategory)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let category = create_category(db, &guild.guild_id, ping_format.id).await?;

        assert_eq!(category.guild_id, guild.guild_id);
        assert_eq!(category.ping_format_id, ping_format.id);
        assert!(!category.name.is_empty());
        assert!(category.ping_cooldown.is_none());
        assert!(category.ping_reminder.is_none());
        assert!(category.max_pre_ping.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn creates_category_with_custom_values() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(FleetCategory)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let category = FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
            .name("Custom Category")
            .ping_cooldown(Some(60))
            .ping_reminder(Some(30))
            .max_pre_ping(Some(180))
            .build()
            .await?;

        assert_eq!(category.name, "Custom Category");
        assert_eq!(category.ping_cooldown, Some(60));
        assert_eq!(category.ping_reminder, Some(30));
        assert_eq!(category.max_pre_ping, Some(180));

        Ok(())
    }

    #[tokio::test]
    async fn creates_multiple_unique_categories() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(FleetCategory)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let category1 = create_category(db, &guild.guild_id, ping_format.id).await?;
        let category2 = create_category(db, &guild.guild_id, ping_format.id).await?;

        assert_ne!(category1.id, category2.id);
        assert_ne!(category1.name, category2.name);
        assert_eq!(category1.guild_id, category2.guild_id);
        assert_eq!(category1.ping_format_id, category2.ping_format_id);

        Ok(())
    }
}
