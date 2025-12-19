use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, Database, DatabaseConnection};

use crate::error::TestError;

/// Test context containing database connection and test environment setup.
///
/// Provides an in-memory SQLite database connection for isolated unit and integration
/// testing. The database is created lazily on first access and persists for the lifetime
/// of the test context.
pub struct TestContext {
    /// Optional database connection to in-memory SQLite instance.
    ///
    /// Initialized lazily when `database()` is first called. Using `Option` allows
    /// deferred connection until actually needed by the test.
    pub db: Option<DatabaseConnection>,
}

impl TestContext {
    /// Creates a new empty test context.
    ///
    /// Initializes a test context with no database connection. The database connection
    /// will be created lazily when `database()` is first called.
    ///
    /// # Returns
    /// - New `TestContext` instance with no database connection
    pub fn new() -> Self {
        Self { db: None }
    }

    /// Gets or creates the in-memory SQLite database connection.
    ///
    /// Returns a reference to the existing database connection if one exists, otherwise
    /// creates a new in-memory SQLite database and stores the connection. The connection
    /// persists for the lifetime of this test context.
    ///
    /// # Returns
    /// - `Ok(&DatabaseConnection)` - Reference to the database connection
    /// - `Err(TestError::Database)` - Failed to connect to in-memory SQLite database
    pub async fn database(&mut self) -> Result<&DatabaseConnection, TestError> {
        match self.db {
            Some(ref db) => Ok(db),
            None => {
                let db = Database::connect("sqlite::memory:").await?;

                Ok(self.db.insert(db))
            }
        }
    }

    /// Creates database tables from the provided CREATE TABLE statements.
    ///
    /// Executes each CREATE TABLE statement in sequence to set up the required database
    /// schema for the test. Typically called internally by `TestBuilder::build()` rather
    /// than directly.
    ///
    /// # Arguments
    /// - `stmts` - Vector of CREATE TABLE statements to execute
    ///
    /// # Returns
    /// - `Ok(())` - All tables created successfully
    /// - `Err(TestError::Database)` - Failed to create one or more tables (invalid SQL,
    ///   constraint violations, etc.)
    pub async fn with_tables(&mut self, stmts: Vec<TableCreateStatement>) -> Result<(), TestError> {
        let db = self.database().await?;

        for stmt in stmts {
            db.execute(&stmt).await?;
        }

        Ok(())
    }
}
