use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: i32,
    pub name: String,
    pub admin: bool,
}
