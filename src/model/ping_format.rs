use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingFormatDto {
    pub id: i32,
    pub guild_id: i64,
    pub name: String,
    pub fields: Vec<PingFormatFieldDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingFormatFieldDto {
    pub id: i32,
    pub ping_format_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePingFormatDto {
    pub name: String,
    pub fields: Vec<CreatePingFormatFieldDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePingFormatFieldDto {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePingFormatDto {
    pub name: String,
    pub fields: Vec<UpdatePingFormatFieldDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePingFormatFieldDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedPingFormatsDto {
    pub ping_formats: Vec<PingFormatDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
