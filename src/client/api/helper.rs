use crate::{client::model::error::ApiError, model::api::ErrorDto};
use reqwasm::http::{Request, Response};
use serde::de::DeserializeOwned;

/// Helper function to parse API responses with consistent error handling
pub async fn parse_response<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    let status = response.status() as u64;

    if (200..300).contains(&status) {
        response.json::<T>().await.map_err(|e| ApiError {
            status: 500,
            message: format!("Failed to parse response: {}", e),
        })
    } else {
        let message = if let Ok(error_dto) = response.json::<ErrorDto>().await {
            error_dto.error
        } else {
            response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string())
        };

        Err(ApiError { status, message })
    }
}

/// Helper function to parse empty success responses (204 No Content, 201 Created, etc.)
pub async fn parse_empty_response(response: Response) -> Result<(), ApiError> {
    let status = response.status() as u64;

    if (200..300).contains(&status) {
        Ok(())
    } else {
        let message = if let Ok(error_dto) = response.json::<ErrorDto>().await {
            error_dto.error
        } else {
            response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string())
        };

        Err(ApiError { status, message })
    }
}

/// Create a GET request with credentials
pub fn get(url: &str) -> Request {
    Request::get(url).credentials(reqwasm::http::RequestCredentials::Include)
}

/// Create a POST request with credentials and JSON content type
pub fn post(url: &str) -> Request {
    Request::post(url)
        .credentials(reqwasm::http::RequestCredentials::Include)
        .header("Content-Type", "application/json")
}

/// Create a PUT request with credentials and JSON content type
pub fn put(url: &str) -> Request {
    Request::put(url)
        .credentials(reqwasm::http::RequestCredentials::Include)
        .header("Content-Type", "application/json")
}

/// Create a DELETE request with credentials
pub fn delete(url: &str) -> Request {
    Request::delete(url).credentials(reqwasm::http::RequestCredentials::Include)
}

/// Send a request and handle common errors
pub async fn send_request(request: Request) -> Result<Response, ApiError> {
    request.send().await.map_err(|e| ApiError {
        status: 500,
        message: format!("Failed to send request: {}", e),
    })
}

/// Serialize a payload to JSON string
pub fn serialize_json<T: serde::Serialize>(payload: &T) -> Result<String, ApiError> {
    serde_json::to_string(payload).map_err(|e| ApiError {
        status: 500,
        message: format!("Failed to serialize request: {}", e),
    })
}
