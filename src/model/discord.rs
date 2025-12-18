use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct DiscordGuildDto {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub guild_id: u64,
    pub name: String,
    pub icon_hash: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct DiscordGuildRoleDto {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub guild_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub role_id: u64,
    pub name: String,
    pub color: String,
    pub position: i16,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct DiscordGuildChannelDto {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub guild_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub channel_id: u64,
    pub name: String,
    pub position: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PaginatedDiscordGuildRolesDto {
    pub roles: Vec<DiscordGuildRoleDto>,
    pub total: u64,
    pub page: u64,
    pub entries: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PaginatedDiscordGuildChannelsDto {
    pub channels: Vec<DiscordGuildChannelDto>,
    pub total: u64,
    pub page: u64,
    pub entries: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct DiscordGuildMemberDto {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string"
    )]
    pub user_id: u64,
    pub username: String,
    pub display_name: String,
    pub avatar_hash: Option<String>,
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
