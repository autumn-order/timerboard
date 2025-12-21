//! Authentication and authorization middleware.
//!
//! This module provides the `AuthGuard` for enforcing permission-based access control
//! across API endpoints. It validates user sessions, checks database records, and
//! enforces fine-grained permissions including admin access and category-level operations.

use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::server::{
    data::{user::UserRepository, user_category_permission::UserCategoryPermissionRepository},
    error::{auth::AuthError, AppError},
    middleware::session::AuthSession,
    model::user::User,
};

/// Permission levels for access control.
///
/// Defines the different types of permissions that can be checked by the `AuthGuard`.
/// Each variant represents a specific permission level with associated context data.
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    /// Full administrative access to all resources.
    ///
    /// Users with admin permission bypass all other permission checks.
    Admin,

    /// Permission to view a specific fleet category.
    ///
    /// # Fields
    /// - `u64` - Discord guild ID where the category exists
    /// - `i32` - Category ID to check view access for
    CategoryView(u64, i32),

    /// Permission to create timers in a specific fleet category.
    ///
    /// # Fields
    /// - `u64` - Discord guild ID where the category exists
    /// - `i32` - Category ID to check create access for
    CategoryCreate(u64, i32),
}

/// Authentication guard for permission-based access control.
///
/// Validates user sessions, retrieves user data from the database, and enforces
/// permission checks before allowing access to protected resources. Admin users
/// bypass non-admin permission checks.
pub struct AuthGuard<'a> {
    /// Database connection for user and permission lookups.
    db: &'a DatabaseConnection,
    /// User session for retrieving authenticated user ID.
    session: &'a Session,
}

impl<'a> AuthGuard<'a> {
    /// Creates a new authentication guard.
    ///
    /// # Arguments
    /// - `db` - Database connection for user and permission queries
    /// - `session` - User session containing authentication state
    pub fn new(db: &'a DatabaseConnection, session: &'a Session) -> Self {
        Self { db, session }
    }

    /// Enforces permission requirements for the current user.
    ///
    /// Validates that the user is authenticated via session, exists in the database,
    /// and has all required permissions. Admin users automatically pass all non-admin
    /// permission checks. For category-level permissions, queries the database to
    /// verify access rights.
    ///
    /// # Arguments
    /// - `permissions` - Slice of permissions that must all be satisfied
    ///
    /// # Returns
    /// - `Ok(User)` - User has all required permissions
    /// - `Err(AuthError::UserNotInSession)` - No user ID found in session
    /// - `Err(AppError::InternalError(ParseStringId))` - Failed to parse user ID from session
    /// - `Err(AuthError::UserNotInDatabase)` - User ID in session not found in database
    /// - `Err(AuthError::AccessDenied)` - User lacks one or more required permissions
    /// - `Err(DbErr(_))` - Database error during permission checks
    pub async fn require(&self, permissions: &[Permission]) -> Result<User, AppError> {
        let auth_session = AuthSession::new(self.session);
        let user_repo = UserRepository::new(self.db);
        let permission_repo = UserCategoryPermissionRepository::new(self.db);

        let Some(user_id) = auth_session.get_user_id().await? else {
            return Err(AuthError::UserNotInSession.into());
        };

        let Some(user) = user_repo.find_by_id(user_id).await? else {
            return Err(AuthError::UserNotInDatabase(user_id).into());
        };

        for permission in permissions {
            match permission {
                Permission::Admin => {
                    if !user.admin {
                        return Err(AuthError::AccessDenied(
                            user_id,
                            "User attempted to access admin-gated route but did not have sufficient permissions".to_string()
                        ).into());
                    }
                }
                Permission::CategoryView(guild_id, category_id) => {
                    // Admins bypass all permission checks
                    if user.admin {
                        continue;
                    }

                    // Check if user has view access to this category
                    let has_access = permission_repo
                        .user_can_view_category(user_id, *category_id)
                        .await?;

                    if !has_access {
                        return Err(AuthError::AccessDenied(
                            user_id,
                            format!(
                                "User does not have view access to category {} in guild {}",
                                category_id, guild_id
                            ),
                        )
                        .into());
                    }
                }
                Permission::CategoryCreate(guild_id, category_id) => {
                    // Admins bypass all permission checks
                    if user.admin {
                        continue;
                    }

                    // Check if user has create access to this category
                    let has_access = permission_repo
                        .user_can_create_category(user_id, *category_id)
                        .await?;

                    if !has_access {
                        return Err(AuthError::AccessDenied(
                            user_id,
                            format!(
                                "User does not have create access to category {} in guild {}",
                                category_id, guild_id
                            ),
                        )
                        .into());
                    }
                }
            }
        }

        Ok(user)
    }
}
