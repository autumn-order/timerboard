//! Fleet factory for creating test fleet entities.
//!
//! This module provides factory methods for creating fleet entities with
//! sensible defaults, reducing boilerplate in tests. The factory supports
//! customization through a builder pattern.

use crate::factory::helpers::next_id;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};

/// Factory for creating test fleets with customizable fields.
///
/// Provides a builder pattern for creating fleet entities with default
/// values that can be overridden as needed for specific test scenarios.
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::factory::fleet::FleetFactory;
///
/// let fleet = FleetFactory::new(&db, 1, "user_123")
///     .name("Custom Fleet")
///     .description(Some("My fleet".to_string()))
///     .build()
///     .await?;
/// ```
pub struct FleetFactory<'a> {
    db: &'a DatabaseConnection,
    category_id: i32,
    name: String,
    commander_id: String,
    fleet_time: chrono::DateTime<Utc>,
    description: Option<String>,
    hidden: bool,
    disable_reminder: bool,
}

impl<'a> FleetFactory<'a> {
    /// Creates a new FleetFactory with default values.
    ///
    /// Defaults:
    /// - name: `"Fleet {id}"` where id is auto-incremented
    /// - fleet_time: 1 hour from now
    /// - description: `Some("Test fleet description")`
    /// - hidden: `false`
    /// - disable_reminder: `false`
    ///
    /// # Arguments
    /// - `db` - Database connection for inserting the entity
    /// - `category_id` - Fleet category ID this fleet belongs to
    /// - `commander_id` - Discord ID of the fleet commander
    ///
    /// # Returns
    /// - `FleetFactory` - New factory instance with defaults
    pub fn new(
        db: &'a DatabaseConnection,
        category_id: i32,
        commander_id: impl Into<String>,
    ) -> Self {
        let id = next_id();
        Self {
            db,
            category_id,
            name: format!("Fleet {}", id),
            commander_id: commander_id.into(),
            fleet_time: Utc::now() + chrono::Duration::hours(1),
            description: Some("Test fleet description".to_string()),
            hidden: false,
            disable_reminder: false,
        }
    }

    /// Sets the fleet name.
    ///
    /// # Arguments
    /// - `name` - Display name for the fleet
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the fleet time.
    ///
    /// # Arguments
    /// - `fleet_time` - Scheduled time for the fleet
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn fleet_time(mut self, fleet_time: chrono::DateTime<Utc>) -> Self {
        self.fleet_time = fleet_time;
        self
    }

    /// Sets the fleet description.
    ///
    /// # Arguments
    /// - `description` - Optional fleet description
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets whether the fleet is hidden.
    ///
    /// # Arguments
    /// - `hidden` - Whether the fleet should be hidden
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Sets whether reminders are disabled.
    ///
    /// # Arguments
    /// - `disable_reminder` - Whether to disable reminders
    ///
    /// # Returns
    /// - `Self` - Factory instance for method chaining
    pub fn disable_reminder(mut self, disable_reminder: bool) -> Self {
        self.disable_reminder = disable_reminder;
        self
    }

    /// Builds and inserts the fleet entity into the database.
    ///
    /// # Returns
    /// - `Ok(entity::fleet::Model)` - Created fleet entity
    /// - `Err(DbErr)` - Database error during insert
    pub async fn build(self) -> Result<entity::fleet::Model, DbErr> {
        entity::fleet::ActiveModel {
            id: ActiveValue::NotSet,
            category_id: ActiveValue::Set(self.category_id),
            name: ActiveValue::Set(self.name),
            commander_id: ActiveValue::Set(self.commander_id),
            fleet_time: ActiveValue::Set(self.fleet_time),
            description: ActiveValue::Set(self.description),
            hidden: ActiveValue::Set(self.hidden),
            disable_reminder: ActiveValue::Set(self.disable_reminder),
            created_at: ActiveValue::Set(Utc::now()),
        }
        .insert(self.db)
        .await
    }
}

/// Creates a fleet with default values for the specified category and commander.
///
/// Shorthand for `FleetFactory::new(db, category_id, commander_id).build().await`.
///
/// # Arguments
/// - `db` - Database connection
/// - `category_id` - Fleet category ID
/// - `commander_id` - Discord ID of the fleet commander
///
/// # Returns
/// - `Ok(entity::fleet::Model)` - Created fleet entity
/// - `Err(DbErr)` - Database error during insert
///
/// # Example
///
/// ```rust,ignore
/// let fleet = create_fleet(&db, 1, "user_123").await?;
/// ```
pub async fn create_fleet(
    db: &DatabaseConnection,
    category_id: i32,
    commander_id: impl Into<String>,
) -> Result<entity::fleet::Model, DbErr> {
    FleetFactory::new(db, category_id, commander_id)
        .build()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::TestBuilder;
    use crate::factory::discord_guild::create_guild;
    use crate::factory::fleet_category::create_category;
    use crate::factory::ping_format::create_ping_format;
    use crate::factory::user::create_user;
    use entity::prelude::*;

    #[tokio::test]
    async fn creates_fleet_with_defaults() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(User)
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(FleetCategory)
            .with_table(Fleet)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let user = create_user(db).await?;
        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let category = create_category(db, &guild.guild_id, ping_format.id).await?;
        let fleet = create_fleet(db, category.id, &user.discord_id).await?;

        assert_eq!(fleet.category_id, category.id);
        assert_eq!(fleet.commander_id, user.discord_id);
        assert!(!fleet.name.is_empty());
        assert!(fleet.description.is_some());
        assert!(!fleet.hidden);
        assert!(!fleet.disable_reminder);

        Ok(())
    }

    #[tokio::test]
    async fn creates_fleet_with_custom_values() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(User)
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(FleetCategory)
            .with_table(Fleet)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let user = create_user(db).await?;
        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let category = create_category(db, &guild.guild_id, ping_format.id).await?;

        let custom_time = Utc::now() + chrono::Duration::hours(2);
        let fleet = FleetFactory::new(db, category.id, &user.discord_id)
            .name("Custom Fleet")
            .description(Some("Custom description".to_string()))
            .fleet_time(custom_time)
            .hidden(true)
            .disable_reminder(true)
            .build()
            .await?;

        assert_eq!(fleet.name, "Custom Fleet");
        assert_eq!(fleet.description, Some("Custom description".to_string()));
        assert_eq!(fleet.fleet_time, custom_time);
        assert!(fleet.hidden);
        assert!(fleet.disable_reminder);

        Ok(())
    }

    #[tokio::test]
    async fn creates_multiple_unique_fleets() -> Result<(), DbErr> {
        let test = TestBuilder::new()
            .with_table(User)
            .with_table(DiscordGuild)
            .with_table(PingFormat)
            .with_table(FleetCategory)
            .with_table(Fleet)
            .build()
            .await
            .unwrap();
        let db = test.db.as_ref().unwrap();

        let user = create_user(db).await?;
        let guild = create_guild(db).await?;
        let ping_format = create_ping_format(db, &guild.guild_id).await?;
        let category = create_category(db, &guild.guild_id, ping_format.id).await?;

        let fleet1 = create_fleet(db, category.id, &user.discord_id).await?;
        let fleet2 = create_fleet(db, category.id, &user.discord_id).await?;

        assert_ne!(fleet1.id, fleet2.id);
        assert_ne!(fleet1.name, fleet2.name);
        assert_eq!(fleet1.category_id, fleet2.category_id);
        assert_eq!(fleet1.commander_id, fleet2.commander_id);

        Ok(())
    }
}
