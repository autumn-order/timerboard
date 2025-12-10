use crate::server::error::{AppError, ConfigError};

pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?,
        })
    }
}
