use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DiscordGuildDto {
    pub id: i32,
    pub guild_id: i32,
    pub name: String,
    pub icon_hash: Option<String>,
}
