//! Application state shared across all request handlers.
//!
//! This module defines the `AppState` struct which holds all shared resources and
//! dependencies needed by the application. The state is initialized once during startup
//! and then cloned for each request handler through Axum's state extraction.
//!
//! The state includes:
//! - Database connection pool for data persistence
//! - HTTP client for external API requests
//! - OAuth2 client for Discord authentication
//! - Admin code service for temporary admin access
//! - Discord HTTP client for bot operations
//! - Application URL for generating links

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    Client, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, RevocationErrorResponseType,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse,
};
use sea_orm::DatabaseConnection;
use serenity::http::Http;
use std::sync::Arc;

use super::service::admin::code::AdminCodeService;

/// Type alias for the OAuth2 client configured for Discord authentication.
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

/// Application state containing shared resources and dependencies.
///
/// This struct holds all the shared state that needs to be accessible across
/// request handlers. It is initialized once during server startup and then
/// cloned (cheaply, as it contains reference-counted or cloneable types) for
/// each incoming request via Axum's state extraction.
///
/// All fields use cheap-to-clone types:
/// - `DatabaseConnection` is a connection pool (clones share the pool)
/// - `reqwest::Client` uses an `Arc` internally
/// - `OAuth2Client` is designed to be cloned
/// - `AdminCodeService` uses `Arc` for shared state
/// - `Arc<Http>` is a reference-counted pointer
/// - `String` is cloned when needed
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool for accessing persistent storage.
    ///
    /// This connection is shared across all requests and manages a pool of
    /// connections to the SQLite database.
    pub db: DatabaseConnection,

    /// HTTP client for making external API requests.
    ///
    /// Configured with security settings (no redirects) to prevent SSRF
    /// vulnerabilities. Used for Discord API calls and other external services.
    pub http_client: reqwest::Client,

    /// OAuth2 client for Discord authentication flow.
    ///
    /// Handles the OAuth2 authentication flow including generating login URLs
    /// and exchanging authorization codes for access tokens.
    pub oauth_client: OAuth2Client,

    /// Service for managing temporary admin codes.
    ///
    /// Used to generate and validate temporary admin codes that allow the first
    /// user to gain admin access when no admin users exist in the database.
    pub admin_code_service: AdminCodeService,

    /// Discord HTTP client for bot API operations.
    ///
    /// Used by the Discord bot and notification services to send messages,
    /// embeds, and interact with Discord's API.
    pub discord_http: Arc<Http>,

    /// Application base URL for generating links.
    ///
    /// Used to construct full URLs for OAuth2 callbacks, embed links, and
    /// other resources that need to reference the application.
    pub app_url: String,
}

impl AppState {
    /// Creates a new application state with the provided dependencies.
    ///
    /// This constructor is called once during server startup after all
    /// dependencies have been initialized. The resulting state is then
    /// provided to the Axum router for use in request handlers.
    ///
    /// # Arguments
    /// - `db` - Database connection pool
    /// - `http_client` - HTTP client for external API requests
    /// - `oauth_client` - OAuth2 client for Discord authentication
    /// - `admin_code_service` - Service for managing admin codes
    /// - `discord_http` - Discord HTTP client for bot operations
    /// - `app_url` - Application base URL
    ///
    /// # Returns
    /// - `AppState` - Initialized application state ready for use
    pub fn new(
        db: DatabaseConnection,
        http_client: reqwest::Client,
        oauth_client: OAuth2Client,
        admin_code_service: AdminCodeService,
        discord_http: Arc<Http>,
        app_url: String,
    ) -> Self {
        Self {
            db,
            http_client,
            oauth_client,
            admin_code_service,
            discord_http,
            app_url,
        }
    }
}
