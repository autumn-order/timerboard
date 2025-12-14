use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sea_orm::DatabaseConnection;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

use crate::server::data::user::UserRepository;
use crate::server::service::admin::code::AdminCodeService;
use crate::server::state::OAuth2Client;
use crate::server::{config::Config, error::AppError};

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

/// Creates a session manager using SQLite for session storage
///
/// Sets up session handling with SQLite as the backing store. Sessions expire after 7 days
/// of inactivity. Cookie security settings are automatically configured based on build mode
/// (secure cookies in release, non-secure in debug for easier local development).
///
/// # Arguments
/// - `db` - Database connection to use for session storage
///
/// # Returns
/// - `Ok(SessionManagerLayer)` - Configured session manager
/// - `Err(AppError)` - Failed to initialize session store
pub async fn connect_to_session(
    db: &sea_orm::DatabaseConnection,
) -> Result<SessionManagerLayer<SqliteStore>, AppError> {
    use time::Duration;

    // Get the underlying SQLx pool from SeaORM connection
    let pool = db.get_sqlite_connection_pool();
    let session_store = SqliteStore::new(pool.clone());

    // Initialize the session table in the database
    session_store.migrate().await?;

    // Set secure based on build mode: in development (debug) use false, otherwise true.
    let development_mode = cfg!(debug_assertions);
    let secure_cookies = !development_mode;

    let session = SessionManagerLayer::new(session_store)
        .with_secure(secure_cookies)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    Ok(session)
}

pub fn setup_reqwest_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build")
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

/// Checks if any admin users exist in the database and generates an admin login link if none exist.
///
/// When the application starts, this function:
/// 1. Queries the database to check if any admin users exist
/// 2. If no admin users are found:
///    - Generates a temporary admin code (valid for 60 seconds)
///    - Constructs and prints a login link with the admin code to the console
///    - The first user to login with this link will be granted admin privileges
///
/// The admin code is stored in memory and can only be used once. It expires after 60 seconds.
///
/// # Arguments
/// - `db` - Database connection to query for admin users
/// - `config` - Application configuration containing the app URL
/// - `admin_code_service` - Service for generating and managing admin codes
///
/// # Returns
/// - `Ok(())` if the check completes successfully
/// - `Err(AppError)` if the database query fails
pub async fn check_for_admin(
    db: &DatabaseConnection,
    config: &Config,
    admin_code_service: &AdminCodeService,
) -> Result<(), AppError> {
    use dioxus_logger::tracing;

    let user_repo = UserRepository::new(db);
    let has_admin = user_repo.admin_exists().await?;

    if !has_admin {
        let admin_code = admin_code_service.generate().await;
        let login_url = format!(
            "{}/api/auth/login?admin_code={}",
            config.app_url, admin_code
        );

        tracing::info!("Admin Login URL (valid for 60 seconds):\n{}", login_url);
    }

    Ok(())
}
