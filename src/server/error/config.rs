//! Configuration error types.
//!
//! This module defines errors that occur during application startup when loading
//! and validating environment variables. Configuration errors are always fatal
//! and prevent the application from starting.

use thiserror::Error;

/// Configuration error type for environment variable validation failures.
///
/// These errors occur during application startup when the configuration system detects
/// missing or invalid environment variables. Configuration errors are always treated as
/// fatal and result in 500 Internal Server Error responses if encountered during request
/// handling, though typically they prevent the application from starting at all.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Required environment variable is not set.
    ///
    /// The application requires this environment variable to be defined. Check the
    /// documentation or `.env.example` file for required configuration variables.
    ///
    /// # Fields
    /// - Name of the missing environment variable
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
}
