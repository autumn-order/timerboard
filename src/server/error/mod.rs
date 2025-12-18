//! Error types and HTTP response handling.
//!
//! This module provides the application's error hierarchy and conversion logic for
//! transforming errors into appropriate HTTP responses. The `AppError` enum serves
//! as the top-level error type that wraps domain-specific errors and implements
//! `IntoResponse` for automatic error handling in API endpoints.

pub mod auth;
pub mod config;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use dioxus_logger::tracing;
use thiserror::Error;

use crate::{
    model::api::ErrorDto,
    server::error::{auth::AuthError, config::ConfigError},
};

/// Top-level application error type.
///
/// Aggregates all possible error types that can occur in the application and provides
/// automatic conversion to HTTP responses. Most variants use `#[from]` for automatic
/// error conversion. Domain-specific errors like `AuthError` handle their own response
/// mapping, while generic variants provide standard HTTP status codes.
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration error during startup or environment variable loading.
    ///
    /// Always results in 500 Internal Server Error as configuration issues
    /// prevent normal application operation.
    #[error(transparent)]
    ConfigErr(#[from] ConfigError),

    /// Authentication or authorization error.
    ///
    /// Delegates to `AuthError::into_response()` for custom status code mapping
    /// (401 Unauthorized, 403 Forbidden, etc.).
    #[error(transparent)]
    AuthErr(#[from] AuthError),

    /// Database operation error from SeaORM.
    ///
    /// Results in 500 Internal Server Error with error details logged server-side.
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),

    /// SQLx database driver error.
    ///
    /// Results in 500 Internal Server Error with error details logged server-side.
    #[error(transparent)]
    SqlxErr(#[from] sea_orm::SqlxError),

    /// Session store operation error.
    ///
    /// Results in 500 Internal Server Error as session failures prevent
    /// authentication and state management.
    #[error(transparent)]
    SessionErr(#[from] tower_sessions::session::Error),

    /// HTTP client request error from reqwest.
    ///
    /// Results in 500 Internal Server Error when external API calls fail.
    #[error(transparent)]
    ReqwestErr(#[from] reqwest::Error),

    /// Discord API error from Serenity.
    ///
    /// Boxed due to large size. Results in 500 Internal Server Error when
    /// Discord bot operations fail.
    #[error(transparent)]
    DiscordErr(#[from] Box<serenity::Error>),

    /// Cron scheduler error.
    ///
    /// Results in 500 Internal Server Error when scheduled job operations fail.
    #[error(transparent)]
    SchedulerErr(#[from] tokio_cron_scheduler::JobSchedulerError),

    /// Resource not found error.
    ///
    /// Results in 404 Not Found with the provided error message.
    ///
    /// # Fields
    /// - Message describing what resource was not found
    #[error("{0}")]
    NotFound(String),

    /// Invalid request error.
    ///
    /// Results in 400 Bad Request with the provided error message.
    ///
    /// # Fields
    /// - Message describing what was invalid about the request
    #[error("{0}")]
    BadRequest(String),

    /// Internal server error with custom message.
    ///
    /// Results in 500 Internal Server Error. The provided message is logged
    /// but a generic message is returned to the client.
    ///
    /// # Fields
    /// - Detailed error message for server-side logging
    #[error("{0}")]
    InternalError(String),
}

/// Manual conversion from serenity::Error to AppError.
///
/// Boxes the error to reduce the size of the AppError enum, as serenity::Error
/// is very large and would make all AppError variants larger if not boxed.
impl From<serenity::Error> for AppError {
    fn from(err: serenity::Error) -> Self {
        AppError::DiscordErr(Box::new(err))
    }
}

/// Converts application errors into HTTP responses.
///
/// Maps each error variant to an appropriate HTTP status code and response body.
/// Authentication errors delegate to their own response handling, while other errors
/// use standard mappings. Internal errors are logged with full details but return
/// generic messages to avoid information leakage.
///
/// # Returns
/// - 400 Bad Request - For `BadRequest` variant
/// - 404 Not Found - For `NotFound` variant
/// - 500 Internal Server Error - For all other error types (DbErr, SessionErr, etc.)
/// - Variable - For `AuthErr`, delegated to `AuthError::into_response()`
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::AuthErr(err) => err.into_response(),
            Self::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(ErrorDto { error: msg })).into_response()
            }
            Self::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(ErrorDto { error: msg })).into_response()
            }
            Self::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorDto {
                        error: "Internal server error".to_string(),
                    }),
                )
                    .into_response()
            }
            err => InternalServerError(err).into_response(),
        }
    }
}

/// Wrapper type for converting any displayable error into a 500 Internal Server Error response.
///
/// This struct logs the error message and returns a generic "Internal server error" message
/// to the client to avoid leaking implementation details. Used as a fallback for errors that
/// don't have specific HTTP response mappings.
pub struct InternalServerError<E>(pub E);

/// Converts wrapped errors into 500 Internal Server Error responses.
///
/// Logs the full error message for debugging, but returns a generic error message to the
/// client to avoid exposing internal implementation details or sensitive information.
///
/// # Arguments
/// - `E` - Any type that implements `Display` (typically an error type)
///
/// # Returns
/// A 500 Internal Server Error response with a generic error message JSON body
impl<E: std::fmt::Display> IntoResponse for InternalServerError<E> {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: "Internal server error".to_string(),
            }),
        )
            .into_response()
    }
}
