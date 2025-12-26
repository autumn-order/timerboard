//! Ping format field factory for creating test ping format field entities.
//!
//! This module provides factory methods for creating ping format field entities with
//! sensible defaults, reducing boilerplate in tests. The factory supports
//! customization through a builder pattern.

use crate::factory::helpers::next_id;
use crate::fixture;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for creating test ping format fields with customizable fields.
///
/// Provides a builder pattern for creating ping format field entities with default
/// values that can be overridden as needed for specific test scenarios. Default values
/// are sourced from the ping_format_field fixture for consistency across tests.
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
    entity: entity::ping_format_field::Model,
}

impl<'a> PingFormatFieldFactory<'a> {
    /// Creates a new PingFormatFieldFactory with default values from fixture.
    ///
    /// Defaults are sourced from `fixture::ping_format_field::entity()` with a unique
    /// auto-incremented ID to prevent conflicts when creating multiple fields.
    /// The ping_format_id is set to the provided value.
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    /// - `ping_format_id` - ID of the ping format this field belongs to
    ///
    /// # Returns
    /// - `PingFormatFieldFactory` - New factory instance with defaults
    pub fn new(db: &'a DatabaseConnection, ping_format_id: i32) -> Self {
        let id = next_id();
        let entity = fixture::ping_format_field::entity_builder()
            .ping_format_id(ping_format_id)
            .name(format!("Field {}", id))
            .build();

        Self { db, entity }
    }

    /// Sets the field name.
    ///
    /// # Arguments
    /// - `name` - Display name for the field
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.entity.name = name.into();
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
        self.entity.priority = priority;
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
            ping_format_id: ActiveValue::Set(self.entity.ping_format_id),
            name: ActiveValue::Set(self.entity.name),
            priority: ActiveValue::Set(self.entity.priority),
            field_type: ActiveValue::NotSet,
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
