//! Ping format field factory for creating test ping format field entities.
//!
//! This module provides factory methods for creating ping format field entities with
//! sensible defaults, reducing boilerplate in tests. The factory supports
//! customization through a builder pattern.

use crate::factory::helpers::next_id;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for creating test ping format fields with customizable fields.
///
/// Provides a builder pattern for creating ping format field entities with default
/// values that can be overridden as needed for specific test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::factory::ping_format_field::PingFormatFieldFactory;
///
/// let field = PingFormatFieldFactory::new(&db, ping_format_id)
///     .name("Custom Field")
///     .priority(5)
///     .build()
///     .await?;
/// ```
pub struct PingFormatFieldFactory<'a> {
    db: &'a DatabaseConnection,
    ping_format_id: i32,
    name: String,
    priority: i32,
    default_value: Option<String>,
}

impl<'a> PingFormatFieldFactory<'a> {
    /// Creates a new PingFormatFieldFactory with default values.
    ///
    /// Defaults:
    /// - name: `"Field {id}"` where id is auto-incremented
    /// - priority: 1
    /// - default_value: None
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    /// - `ping_format_id` - ID of the ping format this field belongs to
    ///
    /// # Returns
    /// - `PingFormatFieldFactory` - New factory instance with defaults
    pub fn new(db: &'a DatabaseConnection, ping_format_id: i32) -> Self {
        let id = next_id();
        Self {
            db,
            ping_format_id,
            name: format!("Field {}", id),
            priority: 1,
            default_value: None,
        }
    }

    /// Sets the field name.
    ///
    /// # Arguments
    /// - `name` - Display name for the field
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the field priority (order).
    ///
    /// # Arguments
    /// - `priority` - Sort priority for the field
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the default value for the field.
    ///
    /// # Arguments
    /// - `default_value` - Optional default value
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn default_value(mut self, default_value: Option<String>) -> Self {
        self.default_value = default_value;
        self
    }

    /// Builds and inserts the ping format field entity into the database.
    ///
    /// # Returns
    /// - `Ok(entity::ping_format_field::Model)` - Created ping format field entity
    /// - `Err(DbErr)` - Database error during insert
    pub async fn build(self) -> Result<entity::ping_format_field::Model, DbErr> {
        entity::ping_format_field::ActiveModel {
            id: ActiveValue::NotSet,
            ping_format_id: ActiveValue::Set(self.ping_format_id),
            name: ActiveValue::Set(self.name),
            priority: ActiveValue::Set(self.priority),
            default_value: ActiveValue::Set(self.default_value),
        }
        .insert(self.db)
        .await
    }
}

/// Creates a ping format field with default values for the specified ping format.
///
/// Shorthand for `PingFormatFieldFactory::new(db, ping_format_id).build().await`.
///
/// # Arguments
/// - `db` - Database connection
/// - `ping_format_id` - ID of the ping format
/// - `name` - Name for the field
/// - `priority` - Sort priority for the field
///
/// # Returns
/// - `Ok(entity::ping_format_field::Model)` - Created ping format field entity
/// - `Err(DbErr)` - Database error during insert
///
/// # Example
///
/// ```rust,ignore
/// let field = create_ping_format_field(&db, format_id, "Location", 1).await?;
/// ```
pub async fn create_ping_format_field(
    db: &DatabaseConnection,
    ping_format_id: i32,
    name: impl Into<String>,
    priority: i32,
) -> Result<entity::ping_format_field::Model, DbErr> {
    PingFormatFieldFactory::new(db, ping_format_id)
        .name(name)
        .priority(priority)
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
    async fn creates_field_with_defaults() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(PingFormatField)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let field = create_ping_format_field(db, ping_format.id, "Test Field", 1).await?;

        assert_eq!(field.ping_format_id, ping_format.id);
        assert_eq!(field.name, "Test Field");
        assert_eq!(field.priority, 1);
        assert!(field.default_value.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn creates_field_with_custom_values() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(PingFormatField)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let field = PingFormatFieldFactory::new(db, ping_format.id)
            .name("Custom Field")
            .priority(5)
            .default_value(Some("Default Value".to_string()))
            .build()
            .await?;

        assert_eq!(field.ping_format_id, ping_format.id);
        assert_eq!(field.name, "Custom Field");
        assert_eq!(field.priority, 5);
        assert_eq!(field.default_value, Some("Default Value".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn creates_multiple_unique_fields() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(PingFormatField)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let field1 = create_ping_format_field(db, ping_format.id, "Field 1", 1).await?;
        let field2 = create_ping_format_field(db, ping_format.id, "Field 2", 2).await?;

        assert_ne!(field1.id, field2.id);
        assert_eq!(field1.ping_format_id, field2.ping_format_id);
        assert_ne!(field1.priority, field2.priority);

        Ok(())
    }
}
