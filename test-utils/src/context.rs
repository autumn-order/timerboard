use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, Database, DatabaseConnection};
use std::sync::Arc;
use time::Duration;
use tower_sessions::{Expiry, Session};
use tower_sessions_sqlx_store::SqliteStore;

use crate::error::TestError;

/// Test context containing database connection, session, and test environment setup.
///
/// Provides an in-memory SQLite database connection and session for isolated
/// unit and integration testing. Both the database and session are created lazily on first
/// access and persist for the lifetime of the test context.
pub struct TestContext {
    /// Optional database connection to in-memory SQLite instance.
    ///
    /// Initialized lazily when `database()` is first called. Using `Option` allows
    /// deferred connection until actually needed by the test.
    pub db: Option<DatabaseConnection>,

    /// Optional session instance for session handling.
    ///
    /// Initialized lazily when `session()` is first called. Uses the same
    /// in-memory SQLite database as `db` for session storage.
    pub session: Option<Session>,
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
        Self {
            db: None,
            session: None,
        }
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

                let db_ref = self.db.insert(db);

                Ok(&*db_ref) // Re-borrow as immutable
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

    /// Gets or creates the test session instance.
    ///
    /// Returns a reference to the existing session if one exists, otherwise
    /// creates a new session backed by the in-memory SQLite database. The session
    /// persists for the lifetime of this test context.
    ///
    /// On first call, this method will:
    /// 1. Initialize the database connection if not already done
    /// 2. Create and migrate the session store table
    /// 3. Create a new session instance
    ///
    /// Subsequent calls return the same session instance.
    ///
    /// # Returns
    /// - `Ok(&Session)` - Reference to the session instance
    /// - `Err(TestError::Database)` - Failed to initialize database connection or session table
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut test = TestContext::new();
    /// let session = test.session().await?;
    ///
    /// // Use session in tests
    /// session.insert("user_id", 123).await?;
    /// ```
    pub async fn session(&mut self) -> Result<&Session, TestError> {
        match self.session {
            Some(ref session) => Ok(session),
            None => {
                // Ensure database is initialized first
                let db = self.database().await?;

                // Get the underlying SQLx pool from SeaORM connection
                let pool = db.get_sqlite_connection_pool();
                let session_store = SqliteStore::new(pool.clone());

                // Initialize the session table in the database
                session_store
                    .migrate()
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                // Create a new session instance with the store
                // Session::new requires: id (None for new), store (Arc), expiry (None for default)
                let session = Session::new(
                    None,
                    Arc::new(session_store),
                    Some(Expiry::OnInactivity(Duration::days(7))),
                );

                let session_ref = self.session.insert(session);

                Ok(&*session_ref) // Re-borrow as immutable
            }
        }
    }

    /// Gets or creates both database and session references.
    ///
    /// Convenience method for tests that need both database and session access.
    /// Initializes both if they don't exist, then returns immutable references to both.
    /// This avoids borrow checker issues when calling `database()` and `session()` separately.
    ///
    /// # Returns
    /// - `Ok((&DatabaseConnection, &Session))` - References to both database and session
    /// - `Err(TestError::Database)` - Failed to initialize database or session
    pub async fn db_and_session(&mut self) -> Result<(&DatabaseConnection, &Session), TestError> {
        // Initialize both (these methods are idempotent)
        self.database().await?;
        self.session().await?;

        // Now return immutable references to the initialized fields
        // This works because we've dropped the mutable borrows from above
        Ok((self.db.as_ref().unwrap(), self.session.as_ref().unwrap()))
    }
}
