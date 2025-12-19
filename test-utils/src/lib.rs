//! Timerboard Test Utils
//!
//! Provides shared testing utilities for building integration and unit tests for the timerboard
//! application. This crate offers a builder pattern for creating test contexts with in-memory
//! SQLite databases and customizable table schemas.
//!
//! # Overview
//!
//! The test utilities consist of three main components:
//! - **TestBuilder**: Fluent builder for configuring test environments
//! - **TestContext**: Test environment containing database connection and setup
//! - **TestError**: Error types that can occur during test setup
//!
//! # Usage
//!
//! Use `TestBuilder` to create a test context with the required database tables:
//!
//! ```rust,ignore
//! use test_utils::builder::TestBuilder;
//! use entity::prelude::User;
//!
//! #[tokio::test]
//! async fn test_user_operations() -> Result<(), TestError> {
//!     let test = TestBuilder::new()
//!         .with_table(User)
//!         .build()
//!         .await?;
//!
//!     let db = test.db.unwrap();
//!     // Perform database operations...
//!
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod context;
pub mod error;
pub mod factory;
pub mod serenity;
