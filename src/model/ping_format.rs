use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingFormatDto {
    pub id: i32,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub guild_id: u64,
    pub name: String,
    pub fields: Vec<PingFormatFieldDto>,
    pub fleet_category_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PingFormatFieldDto {
    pub id: i32,
    pub ping_format_id: i32,
    pub name: String,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePingFormatDto {
    pub name: String,
    pub fields: Vec<CreatePingFormatFieldDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePingFormatFieldDto {
    pub name: String,
    pub priority: i32,
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
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedPingFormatsDto {
    pub ping_formats: Vec<PingFormatDto>,
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
