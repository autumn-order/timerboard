use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorDto {
    pub error: String,
}

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize)]
pub struct SuccessDto {
    pub success: bool,
}
