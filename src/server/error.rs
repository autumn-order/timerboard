use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    /// Required environment variable is not set.
    ///
    /// The application requires this environment variable to be defined. Check the
    /// documentation or `.env.example` file for required configuration variables.
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
}
