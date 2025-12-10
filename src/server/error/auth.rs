use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::model::api::ErrorDto;

#[derive(Error, Debug)]
pub enum AuthError {
    /// CSRF state validation failed during OAuth callback.
    ///
    /// The CSRF state token in the OAuth callback URL does not match the token stored
    /// in the session, indicating a potential CSRF attack or an invalid callback request.
    /// Results in a 400 Bad Request response.
    #[error("Failed to login user due to CSRF state mismatch")]
    CsrfValidationFailed,
}

/// Converts authentication errors into HTTP responses.
///
/// Maps authentication errors to appropriate HTTP status codes and user-friendly error messages:
/// - `UserNotInSession` / `UserNotInDatabase` → 404 Not Found with "User not found"
/// - `CsrfValidationFailed` / `CsrfMissingValue` → 400 Bad Request with "There was an issue logging you in"
/// - `CharacterOwnedByAnotherUser` / `CharacterNotOwned` → 400 Bad Request with "Invalid character selection"
/// - Other errors → 500 Internal Server Error with generic message
///
/// All errors are logged at debug level for diagnostics while keeping client-facing messages
/// generic to avoid information leakage.
///
/// # Returns
/// - 400 Bad Request - For CSRF failures and invalid character operations
/// - 404 Not Found - For missing users
/// - 500 Internal Server Error - For unexpected authentication errors
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::CsrfValidationFailed => (
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: "There was an issue logging you in, please try again.".to_string(),
                }),
            )
                .into_response(),
        }
    }
}
