use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateFleetDto {
    pub category_id: i32,
    pub name: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub commander_id: u64,
    pub fleet_time: String, // Format: "YYYY-MM-DD HH:MM" in UTC
    pub description: Option<String>,
    pub field_values: HashMap<i32, String>, // field_id -> value
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub disable_reminder: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdateFleetDto {
    pub category_id: i32,
    pub name: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub commander_id: u64,
    pub fleet_time: String, // Format: "YYYY-MM-DD HH:MM" in UTC or "now"
    pub description: Option<String>,
    pub field_values: HashMap<i32, String>, // field_id -> value
    pub hidden: bool,
    pub disable_reminder: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct FleetDto {
    pub id: i32,
    pub category_id: i32,
    pub category_name: String,
    pub name: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub commander_id: u64,
    pub commander_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub fleet_time: DateTime<Utc>,
    pub description: Option<String>,
    pub field_values: HashMap<String, String>, // field_name -> value
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub hidden: bool,
    pub disable_reminder: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct FleetListItemDto {
    pub id: i32,
    pub category_id: i32,
    pub category_name: String,
    pub name: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub commander_id: u64,
    pub commander_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub fleet_time: DateTime<Utc>,
    pub hidden: bool,
    pub disable_reminder: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PaginatedFleetsDto {
    pub fleets: Vec<FleetListItemDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(D::Error::custom)
}
