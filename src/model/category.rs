use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetCategoryAccessRoleDto {
    #[serde(
        serialize_with = "serialize_i64_as_string",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub role_id: i64,
    pub role_name: String,
    pub role_color: String,
    pub position: i16,
    pub can_view: bool,
    pub can_create: bool,
    pub can_manage: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetCategoryPingRoleDto {
    #[serde(
        serialize_with = "serialize_i64_as_string",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub role_id: i64,
    pub role_name: String,
    pub role_color: String,
    pub position: i16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetCategoryChannelDto {
    #[serde(
        serialize_with = "serialize_i64_as_string",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub channel_id: i64,
    pub channel_name: String,
    pub position: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetCategoryDto {
    pub id: i32,
    #[serde(
        serialize_with = "serialize_i64_as_string",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub guild_id: i64,
    pub ping_format_id: i32,
    pub ping_format_name: String,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<FleetCategoryAccessRoleDto>,
    pub ping_roles: Vec<FleetCategoryPingRoleDto>,
    pub channels: Vec<FleetCategoryChannelDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetCategoryListItemDto {
    pub id: i32,
    #[serde(
        serialize_with = "serialize_i64_as_string",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub guild_id: i64,
    pub ping_format_id: i32,
    pub ping_format_name: String,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles_count: usize,
    pub ping_roles_count: usize,
    pub channels_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFleetCategoryDto {
    pub ping_format_id: i32,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<FleetCategoryAccessRoleDto>,
    pub ping_roles: Vec<FleetCategoryPingRoleDto>,
    pub channels: Vec<FleetCategoryChannelDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFleetCategoryDto {
    pub ping_format_id: i32,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<FleetCategoryAccessRoleDto>,
    pub ping_roles: Vec<FleetCategoryPingRoleDto>,
    pub channels: Vec<FleetCategoryChannelDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedFleetCategoriesDto {
    pub categories: Vec<FleetCategoryListItemDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

fn serialize_i64_as_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)?
        .parse::<i64>()
        .map_err(D::Error::custom)
}
