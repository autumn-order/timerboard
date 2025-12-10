use oauth2::basic::{BasicClient, BasicErrorResponseType, BasicTokenType};
use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    RedirectUrl, RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenUrl,
};

use crate::server::{config::Config, error::AppError};

pub(crate) type OAuth2Client = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

/// Connects to the Sqlite database and runs pending migrations.
///
/// Establishes a connection pool to the Sqlite database using the connection string from
/// configuration, then automatically runs all pending SeaORM migrations to ensure the database
/// schema is up-to-date. This function must complete successfully before the application can
/// access the database.
///
/// # Arguments
/// - `config` - Application configuration containing the database URL
///
/// # Returns
/// - `Ok(DatabaseConnection)` - Connected database with migrations applied
/// - `Err(Error)` - Failed to connect to database or run migrations
pub async fn connect_to_database(config: &Config) -> Result<sea_orm::DatabaseConnection, AppError> {
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{ConnectOptions, Database};

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await?;

    Migrator::up(&db, None).await?;

    Ok(db)
}

/// Setup OAuth2 client for Discord login
pub fn setup_oauth_client(config: &Config) -> OAuth2Client {
    let client_id = ClientId::new(config.discord_client_id.to_string());
    let client_secret = ClientSecret::new(config.discord_client_secret.to_string());
    let auth_url = AuthUrl::new(config.discord_auth_url.to_string()).unwrap();
    let token_url = TokenUrl::new(config.discord_token_url.to_string()).unwrap();
    let redirect_url = RedirectUrl::new(config.discord_redirect_url.to_string()).unwrap();

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
}
