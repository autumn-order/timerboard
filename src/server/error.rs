use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use dioxus_logger::tracing;
use thiserror::Error;

use crate::model::api::ErrorDto;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    /// Required environment variable is not set.
    ///
    /// The application requires this environment variable to be defined. Check the
    /// documentation or `.env.example` file for required configuration variables.
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
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
