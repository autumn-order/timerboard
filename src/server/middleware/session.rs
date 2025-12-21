//! Type-safe session management wrappers.
//!
//! This module provides type-safe interfaces for managing different aspects of user sessions,
//! organized by concern. Each struct handles a specific domain of session data, preventing
//! typos, ensuring type consistency, and centralizing session-related logic.
//!
//! # Architecture
//!
//! Session management is split into focused concerns:
//! - `AuthSession` - User authentication state (user ID)
//! - `CsrfSession` - CSRF token management for OAuth flows
//! - `OAuthFlowSession` - Temporary OAuth flow state (admin codes, bot addition)
//!
//! Each struct wraps the same underlying `Session` but exposes only the methods
//! relevant to its concern, following the Interface Segregation Principle.

use tower_sessions::Session;

use crate::server::{error::AppError, util::parse::parse_u64_from_string};

// Session key constants
const SESSION_AUTH_USER_ID: &str = "auth:user";
const SESSION_AUTH_CSRF_TOKEN: &str = "auth:csrf_token";
const SESSION_AUTH_SET_ADMIN: &str = "auth:set_admin";
const SESSION_AUTH_ADDING_BOT: &str = "auth:adding_bot";

/// Authentication session management.
///
/// Handles user authentication state including storing and retrieving the
/// authenticated user's Discord ID and session lifecycle operations.
pub struct AuthSession<'a> {
    /// The underlying tower-sessions Session instance.
    session: &'a Session,
}

impl<'a> AuthSession<'a> {
    /// Creates a new AuthSession wrapper.
    ///
    /// # Arguments
    /// - `session` - Reference to the tower-sessions Session to wrap
    ///
    /// # Returns
    /// A new AuthSession instance
    pub fn new(session: &'a Session) -> Self {
        Self { session }
    }

    /// Gets the underlying Session reference for use with extractors.
    ///
    /// This is useful when you need to pass the raw Session to other APIs
    /// that expect it directly, such as `AuthGuard`.
    ///
    /// # Returns
    /// Reference to the underlying Session
    pub fn inner(&self) -> &Session {
        self.session
    }

    /// Stores the user's Discord ID in the session.
    ///
    /// Called after successful authentication to establish a logged-in session.
    ///
    /// # Arguments
    /// - `user_id` - The user's Discord ID as a string
    ///
    /// # Returns
    /// - `Ok(())` - User ID successfully stored
    /// - `Err(AppError::SessionErr(_))` - Failed to store in session
    pub async fn set_user_id(&self, user_id: u64) -> Result<(), AppError> {
        self.session
            .insert(SESSION_AUTH_USER_ID, user_id.to_string())
            .await?;
        Ok(())
    }

    /// Retrieves the user's Discord ID from the session.
    ///
    /// Used to identify the currently authenticated user.
    ///
    /// # Returns
    /// - `Ok(Some(user_id))` - User is logged in, returns their Discord ID
    /// - `Ok(None)` - No user in session (not logged in)
    /// - `Err(AppError::SessionErr(_))` - Failed to access session
    pub async fn get_user_id(&self) -> Result<Option<u64>, AppError> {
        let Some(user_id_str) = self.session.get::<String>(SESSION_AUTH_USER_ID).await? else {
            return Ok(None);
        };

        let user_id = parse_u64_from_string(user_id_str)?;

        Ok(Some(user_id))
    }

    /// Checks if a user is currently logged in.
    ///
    /// Convenience method that returns a boolean instead of an optional user ID.
    ///
    /// # Returns
    /// - `Ok(true)` - User is logged in
    /// - `Ok(false)` - No user in session
    /// - `Err(AppError::SessionErr(_))` - Failed to access session
    pub async fn is_authenticated(&self) -> Result<bool, AppError> {
        Ok(self.get_user_id().await?.is_some())
    }

    /// Clears all data from the session.
    ///
    /// Used during logout to remove all session data including authentication
    /// state and any temporary OAuth flow data.
    pub async fn clear(&self) {
        self.session.clear().await;
    }
}

/// CSRF protection session management.
///
/// Handles CSRF token storage and validation for OAuth flows. Tokens are stored
/// during login initiation and validated during the OAuth callback.
pub struct CsrfSession<'a> {
    /// The underlying tower-sessions Session instance.
    session: &'a Session,
}

impl<'a> CsrfSession<'a> {
    /// Creates a new CsrfSession wrapper.
    ///
    /// # Arguments
    /// - `session` - Reference to the tower-sessions Session to wrap
    ///
    /// # Returns
    /// A new CsrfSession instance
    pub fn new(session: &'a Session) -> Self {
        Self { session }
    }

    /// Stores a CSRF token in the session.
    ///
    /// Used during OAuth flow initiation to store a random token that will
    /// be validated during the callback to prevent CSRF attacks.
    ///
    /// # Arguments
    /// - `token` - The CSRF token to store
    ///
    /// # Returns
    /// - `Ok(())` - Token successfully stored
    /// - `Err(AppError::SessionErr(_))` - Failed to store in session
    pub async fn set_token(&self, token: String) -> Result<(), AppError> {
        self.session.insert(SESSION_AUTH_CSRF_TOKEN, token).await?;
        Ok(())
    }

    /// Retrieves and removes the CSRF token from the session.
    ///
    /// This is used during OAuth callback validation. The token is removed
    /// to prevent replay attacks - each token can only be used once.
    ///
    /// # Returns
    /// - `Ok(Some(token))` - CSRF token was found and removed
    /// - `Ok(None)` - No CSRF token in session
    /// - `Err(AppError::SessionErr(_))` - Failed to access session
    pub async fn take_token(&self) -> Result<Option<String>, AppError> {
        let token = self.session.remove(SESSION_AUTH_CSRF_TOKEN).await?;
        Ok(token)
    }
}

/// OAuth flow state session management.
///
/// Handles temporary state flags for OAuth flows that need to persist across
/// the redirect to Discord and back. This includes admin code validation status
/// and bot addition flow indicators.
pub struct OAuthFlowSession<'a> {
    /// The underlying tower-sessions Session instance.
    session: &'a Session,
}

impl<'a> OAuthFlowSession<'a> {
    /// Creates a new OAuthFlowSession wrapper.
    ///
    /// # Arguments
    /// - `session` - Reference to the tower-sessions Session to wrap
    ///
    /// # Returns
    /// A new OAuthFlowSession instance
    pub fn new(session: &'a Session) -> Self {
        Self { session }
    }

    /// Marks that an admin code was successfully validated during login.
    ///
    /// This flag is checked during the OAuth callback to grant admin privileges
    /// to the newly authenticated user.
    ///
    /// # Arguments
    /// - `set_admin` - Whether to grant admin privileges
    ///
    /// # Returns
    /// - `Ok(())` - Flag successfully stored
    /// - `Err(AppError::SessionErr(_))` - Failed to store in session
    pub async fn set_admin_flag(&self, set_admin: bool) -> Result<(), AppError> {
        self.session
            .insert(SESSION_AUTH_SET_ADMIN, set_admin)
            .await?;
        Ok(())
    }

    /// Retrieves and removes the admin flag from the session.
    ///
    /// Used during OAuth callback to determine if admin privileges should
    /// be granted to the user. The flag is removed to prevent reuse.
    ///
    /// # Returns
    /// - `Ok(true)` - Admin code was validated, grant admin privileges
    /// - `Ok(false)` - No admin code was validated
    /// - `Err(AppError::SessionErr(_))` - Failed to access session
    pub async fn take_admin_flag(&self) -> Result<bool, AppError> {
        let set_admin = self
            .session
            .remove(SESSION_AUTH_SET_ADMIN)
            .await?
            .unwrap_or(false);
        Ok(set_admin)
    }

    /// Marks that this OAuth flow is for adding a bot to a server.
    ///
    /// Used to differentiate between user login flows and bot addition flows,
    /// since they use the same OAuth callback endpoint but have different behavior.
    ///
    /// # Arguments
    /// - `adding_bot` - Whether this is a bot addition flow
    ///
    /// # Returns
    /// - `Ok(())` - Flag successfully stored
    /// - `Err(AppError::SessionErr(_))` - Failed to store in session
    pub async fn set_adding_bot_flag(&self, adding_bot: bool) -> Result<(), AppError> {
        self.session
            .insert(SESSION_AUTH_ADDING_BOT, adding_bot)
            .await?;
        Ok(())
    }

    /// Retrieves and removes the bot addition flag from the session.
    ///
    /// Used during OAuth callback to determine if this is a bot addition
    /// flow versus a user login flow. The flag is removed after reading.
    ///
    /// # Returns
    /// - `Ok(true)` - This is a bot addition flow
    /// - `Ok(false)` - This is a regular user login flow
    /// - `Err(AppError::SessionErr(_))` - Failed to access session
    pub async fn take_adding_bot_flag(&self) -> Result<bool, AppError> {
        let adding_bot = self
            .session
            .remove(SESSION_AUTH_ADDING_BOT)
            .await?
            .unwrap_or(false);
        Ok(adding_bot)
    }
}
