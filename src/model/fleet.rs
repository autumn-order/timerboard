use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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
