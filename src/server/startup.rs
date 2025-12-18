//! Server startup utilities for initializing application dependencies.
//!
//! This module provides functions for setting up the core infrastructure required
//! by the application, including:
//! - Database connection and migration
//! - Session storage configuration
//! - HTTP client setup
//! - OAuth2 client configuration
//! - Admin user initialization
//!
//! All setup functions are called during server startup and must complete successfully
//! before the application can accept requests.

use dioxus_logger::tracing;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sea_orm::DatabaseConnection;
use time::Duration;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

use crate::server::data::user::UserRepository;
use crate::server::service::admin::code::AdminCodeService;
use crate::server::state::OAuth2Client;
use crate::server::{config::Config, error::AppError};

/// Session expiration duration in days.
///
/// Sessions will expire after this many days of inactivity. After expiration,
/// users must log in again to establish a new session.
static SESSION_EXPIRY_DAYS: i64 = 7;

/// Admin code validity duration in seconds.
///
/// When no admin users exist, a temporary admin code is generated that is
/// valid for this many seconds. The first user to log in with this code
/// within the time window will be granted admin privileges.
static ADMIN_CODE_VALIDITY_SECONDS: u64 = 60;

/// Connects to the SQLite database and runs pending migrations.
///
/// Establishes a connection pool to the SQLite database using the connection string
/// from configuration, then automatically runs all pending SeaORM migrations to ensure
/// the database schema is up-to-date. SQL query logging is disabled to reduce noise.
///
/// This function must complete successfully before the application can access the
/// database. Migration failures will prevent the application from starting.
///
/// # Arguments
/// - `config` - Application configuration containing the database URL
///
/// # Returns
/// - `Ok(DatabaseConnection)` - Connected database with all migrations applied
/// - `Err(DbErr(_))` - Failed to connect to database
/// - `Err(DbErr(_))` - Failed to run migrations
pub async fn connect_to_database(config: &Config) -> Result<DatabaseConnection, AppError> {
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{ConnectOptions, Database};

    tracing::info!("Connecting to database: {}", config.database_url);

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await?;

    tracing::info!("Running database migrations");
    Migrator::up(&db, None).await?;

    tracing::info!("Database connection established successfully");

    Ok(db)
}

/// Creates a session manager using SQLite for session storage.
///
/// Sets up session handling with SQLite as the backing store. Sessions expire after
/// 7 days of inactivity, requiring users to log in again. Cookie security settings
/// are automatically configured based on build mode:
/// - Release builds: Secure cookies enabled (HTTPS only)
/// - Debug builds: Secure cookies disabled (allows HTTP for local development)
///
/// All cookies use `SameSite::Lax` and `HttpOnly` for security.
///
/// # Arguments
/// - `db` - Database connection to use for session storage table
///
/// # Returns
/// - `Ok(SessionManagerLayer)` - Configured session manager ready for use
/// - `Err(SqlxError(_))` - Failed to initialize session store
/// - `Err(SqlxError(_))` - Failed to migrate session table
pub async fn connect_to_session(
    db: &DatabaseConnection,
) -> Result<SessionManagerLayer<SqliteStore>, AppError> {
    tracing::info!("Initializing session storage");

    // Get the underlying SQLx pool from SeaORM connection
    let pool = db.get_sqlite_connection_pool();
    let session_store = SqliteStore::new(pool.clone());

    // Initialize the session table in the database
    session_store.migrate().await?;

    // Set secure based on build mode: in development (debug) use false, otherwise true
    let development_mode = cfg!(debug_assertions);
    let secure_cookies = !development_mode;

    tracing::info!(
        "Session storage configured (secure_cookies: {}, expiry: {} days)",
        secure_cookies,
        SESSION_EXPIRY_DAYS
    );

    let session = SessionManagerLayer::new(session_store)
        .with_secure(secure_cookies)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(SESSION_EXPIRY_DAYS)));

    Ok(session)
}

/// Sets up the HTTP client for making external API requests.
///
/// Creates a `reqwest` client configured for secure API communication. The client
/// is configured to NOT follow redirects automatically, which prevents Server-Side
/// Request Forgery (SSRF) vulnerabilities where an attacker could redirect requests
/// to internal services.
///
/// This client is used throughout the application for:
/// - Discord API requests (OAuth2, bot operations)
/// - External service integrations
///
/// # Returns
/// - `reqwest::Client` - Configured HTTP client ready for API requests
///
/// # Panics
/// - Panics if the client fails to build (extremely rare, would indicate system issues)
pub fn setup_reqwest_client() -> reqwest::Client {
    tracing::info!("Setting up HTTP client");

    reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP client should build successfully")
}

/// Sets up the OAuth2 client for Discord authentication.
///
/// Creates a configured OAuth2 client for handling Discord login flow. The client
/// is configured with:
/// - Client ID and secret from Discord Developer Portal
/// - Authorization endpoint for initiating login
/// - Token exchange endpoint for exchanging auth codes for tokens
/// - Redirect URL for OAuth2 callback
///
/// This client is used by the authentication service to:
/// - Generate login URLs with proper parameters
/// - Exchange authorization codes for access tokens
/// - Validate and refresh OAuth2 tokens
///
/// # Arguments
/// - `config` - Application configuration containing Discord OAuth2 credentials
///
/// # Returns
/// - `OAuth2Client` - Configured OAuth2 client ready for Discord authentication
///
/// # Panics
/// - Panics if any of the URLs are malformed (would indicate configuration error)
pub fn setup_oauth_client(config: &Config) -> OAuth2Client {
    tracing::info!("Setting up Discord OAuth2 client");

    let client_id = ClientId::new(config.discord_client_id.to_string());
    let client_secret = ClientSecret::new(config.discord_client_secret.to_string());
    let auth_url = AuthUrl::new(config.discord_auth_url.to_string())
        .expect("Discord auth URL should be valid");
    let token_url = TokenUrl::new(config.discord_token_url.to_string())
        .expect("Discord token URL should be valid");
    let redirect_url = RedirectUrl::new(config.discord_redirect_url.to_string())
        .expect("Discord redirect URL should be valid");

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
}

/// Checks if any admin users exist and generates an admin login link if none exist.
///
/// When the application starts, this function:
/// 1. Queries the database to check if any admin users exist
/// 2. If no admin users are found:
///    - Generates a temporary admin code (valid for 60 seconds)
///    - Constructs and logs a login link with the admin code
///    - The first user to login with this link will be granted admin privileges
///
/// The admin code is stored in memory and can only be used once. It expires after
/// 60 seconds. This mechanism ensures that the first user can bootstrap admin access
/// when setting up a new instance, without requiring manual database intervention.
///
/// # Arguments
/// - `db` - Database connection to query for admin users
/// - `config` - Application configuration containing the app URL for generating the link
/// - `admin_code_service` - Service for generating and managing admin codes
///
/// # Returns
/// - `Ok(())` - Admin check completed successfully
/// - `Err(DbErr(_))` - Failed to query for admin users in database
pub async fn check_for_admin(
    db: &DatabaseConnection,
    config: &Config,
    admin_code_service: &AdminCodeService,
) -> Result<(), AppError> {
    tracing::info!("Checking for existing admin users");

    let user_repo = UserRepository::new(db);
    let has_admin = user_repo.admin_exists().await?;

    if !has_admin {
        tracing::warn!("No admin users found, generating temporary admin login link");

        let admin_code = admin_code_service.generate().await;
        let login_url = format!(
            "{}/api/auth/login?admin_code={}",
            config.app_url, admin_code
        );

        tracing::info!(
            "Admin Login URL (valid for {} seconds):\n{}",
            ADMIN_CODE_VALIDITY_SECONDS,
            login_url
        );
    } else {
        tracing::info!("Admin users exist, skipping admin code generation");
    }

    Ok(())
}
