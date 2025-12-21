use std::num::ParseIntError;
use thiserror::Error;

/// Internal issues with the codebase indicating unexpected behavior & possible bugs
#[derive(Error, Debug)]
pub enum InternalError {
    /// Failure to parse id from String
    ///
    /// Results a in 500 Internal Server Error with a generic message returned
    /// to client.
    #[error("Failed to parse ID from String '{value}': {source}")]
    ParseStringId {
        /// The string value that failed to parse
        value: String,
        /// The underlying parse error
        #[source]
        source: ParseIntError,
    },

    /// Failure to convert Unix timestamp to Discord timestamp
    ///
    /// Occurs when a valid Unix timestamp cannot be converted to Discord's
    /// timestamp format, typically due to timestamp being out of range.
    /// Results in a 500 Internal Server Error with a generic message returned
    /// to client.
    #[error("Failed to convert Unix timestamp {timestamp} to Discord timestamp: {reason}")]
    InvalidDiscordTimestamp {
        /// The Unix timestamp that failed to convert
        timestamp: i64,
        /// The reason for conversion failure
        reason: String,
    },
}
