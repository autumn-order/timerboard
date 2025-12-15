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

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigErr(#[from] ConfigError),
    #[error(transparent)]
    AuthErr(#[from] AuthError),
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error(transparent)]
    SqlxErr(#[from] sea_orm::SqlxError),
    #[error(transparent)]
    SessionErr(#[from] tower_sessions::session::Error),
    #[error(transparent)]
    ReqwestErr(#[from] reqwest::Error),
    #[error(transparent)]
    DiscordErr(#[from] serenity::Error),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    InternalError(String),
}

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
