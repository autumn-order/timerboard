use entity::prelude::*;
use sea_orm::{sea_query::TableCreateStatement, EntityTrait, Schema};

use crate::{context::TestContext, error::TestError};

/// Builder for creating test contexts with customizable database schemas.
///
/// Provides a fluent interface for configuring test environments with in-memory SQLite
/// databases. Use the builder pattern to add entity tables, then call `build()` to
/// create the configured test context.
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::builder::TestBuilder;
/// use entity::prelude::{User, Character};
///
/// let test = TestBuilder::new()
///     .with_table(User)
///     .with_table(Character)
///     .build()
///     .await?;
/// ```
pub struct TestBuilder {
    /// Vector of CREATE TABLE statements to execute during database setup.
    ///
    /// Each statement is generated from an entity model using SeaORM's schema builder.
    /// Statements are executed in the order they were added during `build()`.
    tables: Vec<TableCreateStatement>,
}

impl TestBuilder {
    /// Creates a new test builder with no tables configured.
    ///
    /// Initializes an empty builder ready to have entity tables added via `with_table()`.
    /// Chain method calls to configure the test environment before calling `build()`.
    ///
    /// # Returns
    /// - New `TestBuilder` instance with empty table configuration
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }

    /// Adds an entity table to the test database schema.
    ///
    /// Generates a CREATE TABLE statement from the provided SeaORM entity using SQLite
    /// backend syntax. The table will be created when `build()` is called. Chain multiple
    /// calls to add multiple tables. Tables should be added in dependency order (tables
    /// with foreign keys should be added after their referenced tables).
    ///
    /// # Arguments
    /// - `entity` - SeaORM entity model implementing `EntityTrait` to create table for
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn with_table<E: EntityTrait>(mut self, entity: E) -> Self {
        let schema = Schema::new(sea_orm::DbBackend::Sqlite);
        self.tables.push(schema.create_table_from_entity(entity));
        self
    }

    /// Adds all tables required for fleet operations.
    ///
    /// This convenience method adds the following tables in dependency order:
    /// - User
    /// - DiscordGuild
    /// - PingFormat
    /// - FleetCategory
    /// - Fleet
    ///
    /// Use this when testing fleet-related functionality that doesn't involve
    /// fleet messages. For tests involving fleet messages, use `with_fleet_message_tables()`.
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let test = TestBuilder::new()
    ///     .with_fleet_tables()
    ///     .build()
    ///     .await?;
    /// ```
    pub fn with_fleet_tables(self) -> Self {
        self.with_table(User)
            .with_table(DiscordGuild)
            .with_table(DiscordGuildRole)
            .with_table(DiscordGuildChannel)
            .with_table(PingFormat)
            .with_table(PingFormatField)
            .with_table(FleetCategory)
            .with_table(FleetCategoryAccessRole)
            .with_table(FleetCategoryPingRole)
            .with_table(FleetCategoryChannel)
            .with_table(Fleet)
            .with_table(FleetFieldValue)
    }

    /// Adds all tables required for fleet message operations.
    ///
    /// This convenience method adds the following tables in dependency order:
    /// - User
    /// - DiscordGuild
    /// - PingFormat
    /// - FleetCategory
    /// - Fleet
    /// - FleetMessage
    ///
    /// Use this when testing fleet message functionality. This is equivalent to
    /// calling `with_fleet_tables()` followed by `with_table(FleetMessage)`.
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let test = TestBuilder::new()
    ///     .with_fleet_message_tables()
    ///     .build()
    ///     .await?;
    /// ```
    pub fn with_fleet_message_tables(self) -> Self {
        self.with_fleet_tables().with_table(FleetMessage)
    }

    /// Builds and initializes the test context with configured tables.
    ///
    /// Creates an in-memory SQLite database connection and executes all CREATE TABLE
    /// statements that were added via `with_table()`. Tables are created in the order
    /// they were added to the builder.
    ///
    /// # Returns
    /// - `Ok(TestContext)` - Fully initialized test context with database and tables ready
    /// - `Err(TestError::Database)`- Failed to connect to database or create tables
    pub async fn build(self) -> Result<TestContext, TestError> {
        let mut setup = TestContext::new();

        setup.with_tables(self.tables).await?;

        Ok(setup)
    }
}
