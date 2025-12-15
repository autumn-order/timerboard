use crate::{
    client::{
        api::helper::{delete, get, parse_empty_response, parse_response, post, send_request},
        model::error::ApiError,
    },
    model::user::{PaginatedUsersDto, UserDto},
};

pub async fn get_all_users(page: u64, per_page: u64) -> Result<PaginatedUsersDto, ApiError> {
    let url = format!("/api/admin/users?page={}&per_page={}", page, per_page);
    let request = get(&url);
    let response = send_request(request).await?;
    parse_response(response).await
}

pub async fn get_all_admins() -> Result<Vec<UserDto>, ApiError> {
    let request = get("/api/admin/admins");
    let response = send_request(request).await?;
    parse_response(response).await
}

pub async fn add_admin(user_id: u64) -> Result<(), ApiError> {
    let url = format!("/api/admin/admins/{}", user_id);
    let request = post(&url);
    let response = send_request(request).await?;
    parse_empty_response(response).await
}

pub async fn remove_admin(user_id: u64) -> Result<(), ApiError> {
    let url = format!("/api/admin/admins/{}", user_id);
    let request = delete(&url);
    let response = send_request(request).await?;
    parse_empty_response(response).await
}
