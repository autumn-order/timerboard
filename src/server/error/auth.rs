//! Authentication and authorization error types.
//!
//! This module defines errors related to user authentication, OAuth2 flows, and
//! permission validation. Each error variant maps to an appropriate HTTP status code
//! and user-friendly error message when converted to a response.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use oauth2::{basic::BasicErrorResponseType, HttpClientError, StandardErrorResponse};
use thiserror::Error;

use crate::{model::api::ErrorDto, server::error::InternalServerError};

/// Authentication and authorization error type.
///
/// Represents errors that occur during user authentication, OAuth2 callbacks, session
/// validation, and permission checks. Implements `IntoResponse` to automatically convert
/// errors into appropriate HTTP responses with status codes ranging from 400-500.
#[derive(Error, Debug)]
pub enum AuthError {
    /// CSRF state validation failed during OAuth callback.
    ///
    /// The CSRF state token in the OAuth callback URL does not match the token stored
    /// in the session, indicating a potential CSRF attack or an invalid callback request.
    /// Results in a 400 Bad Request response.
    #[error("Failed to login user due to CSRF state mismatch")]
    CsrfValidationFailed,
    /// Admin code validation failed.
    ///
    /// The provided admin code is invalid, expired, or does not match the stored code.
    /// Results in a 403 Forbidden response.
    #[error("Invalid or expired admin code")]
    AdminCodeValidationFailed,

    /// User ID not found in session.
    ///
    /// The request requires an authenticated user but no user ID exists in the session.
    /// This typically occurs when a user accesses a protected endpoint without logging in.
    /// Results in a 404 Not Found response with "User not found" message.
    #[error("User not found in session")]
    UserNotInSession,

    /// User exists in session but not in database.
    ///
    /// The user ID from the session does not correspond to any user record in the database.
    /// This can occur if a user was deleted while having an active session.
    /// Results in a 404 Not Found response with "User not found" message.
    ///
    /// # Fields
    /// - Discord user ID from the session
    #[error("User {0} not found in database")]
    UserNotInDatabase(u64),

    /// User lacks required permissions for the requested operation.
    ///
    /// The authenticated user does not have sufficient permissions to perform the
    /// requested action. The reason field provides details about the specific permission
    /// that was denied. Results in a 403 Forbidden response.
    ///
    /// # Fields
    /// - `u64` - Discord user ID of the user denied access
    /// - `String` - Detailed reason for access denial (logged but not sent to client)
    #[error("Access denied for user {0}: {1}")]
    AccessDenied(u64, String),

    /// OAuth2 token exchange failed during callback.
    ///
    /// The authorization code from the OAuth2 callback could not be exchanged for
    /// an access token. This typically indicates an issue with the OAuth2 provider
    /// or an invalid/expired authorization code. Results in a 500 Internal Server Error.
    #[error(transparent)]
    RequestTokenErr(
        #[from]
        oauth2::RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ),
}

/// Converts authentication errors into HTTP responses.
///
/// Maps authentication errors to appropriate HTTP status codes and user-friendly error messages.
/// Error details are logged server-side while client-facing messages remain generic to prevent
/// information leakage about system internals or security mechanisms.
///
/// # Returns
/// - `400 Bad Request` - For CSRF validation failures
/// - `403 Forbidden` - For admin code failures and access denied errors
/// - `404 Not Found` - For missing users (both session and database)
/// - `500 Internal Server Error` - For OAuth2 token errors and unexpected failures
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let user_not_found = (
            StatusCode::NOT_FOUND,
            Json(ErrorDto {
                error: "User not found".to_string(),
            }),
        )
            .into_response();

        match self {
            Self::CsrfValidationFailed => (
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: "There was an issue logging you in, please try again.".to_string(),
                }),
            )
                .into_response(),
            Self::UserNotInSession => user_not_found,
            Self::UserNotInDatabase(_) => user_not_found,
            Self::AdminCodeValidationFailed => (
                StatusCode::FORBIDDEN,
                Json(ErrorDto {
                    error: "Invalid or expired admin code.".to_string(),
                }),
            )
                .into_response(),
            Self::AccessDenied(_, _) => (
                StatusCode::FORBIDDEN,
                Json(ErrorDto {
                    error: "Insufficient permissions".to_string(),
                }),
            )
                .into_response(),
            err => InternalServerError(err).into_response(),
        }
    }
}
