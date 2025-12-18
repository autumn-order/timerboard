use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct ErrorDto {
    pub error: String,
}

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SuccessDto {
    pub success: bool,
}
