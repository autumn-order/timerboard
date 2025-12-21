//! User domain models and parameters.
//!
//! Provides domain models for application users with Discord identity and permission
//! tracking. Includes parameter types for user creation and updates during authentication
//! and synchronization operations.

use chrono::{DateTime, Utc};

use crate::{
    model::user::{PaginatedUsersDto, UserDto},
    server::{error::AppError, util::parse::parse_u64_from_string},
};

/// User with Discord identity, permissions, and sync metadata.
///
/// Tracks user's Discord ID, display name, admin privileges, and when their
/// guild memberships and role assignments were last synchronized.
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    /// Discord ID of the user
    pub discord_id: u64,
    /// Display name of the user.
    pub name: String,
    /// Whether the user has admin privileges.
    pub admin: bool,
    /// Last time the user's guild memberships were synchronized.
    pub last_guild_sync_at: DateTime<Utc>,
    /// Last time the user's role memberships were synchronized.
    pub last_role_sync_at: DateTime<Utc>,
}

impl User {
    /// Converts the user domain model to a DTO for API responses.
    ///
    /// Parses the stored String discord_id into u64 for the DTO. If parsing fails,
    /// defaults to 0 (though this should never happen with valid database data).
    ///
    /// # Returns
    /// - `UserDto` - The converted user DTO with discord_id as u64
    pub fn into_dto(self) -> UserDto {
        UserDto {
            discord_id: self.discord_id,
            name: self.name,
            admin: self.admin,
        }
    }

    /// Converts an entity model to a user domain model at the repository boundary.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `Ok(User)` - The converted user domain model
    /// - `Err(AppError::InternalError(ParseStringId))` - Failed to convert stored user
    ///   Discord ID from String to u64
    pub fn from_entity(entity: entity::user::Model) -> Result<Self, AppError> {
        let discord_id = parse_u64_from_string(entity.discord_id)?;

        Ok(Self {
            discord_id: discord_id,
            name: entity.name,
            admin: entity.admin,
            last_guild_sync_at: entity.last_guild_sync_at,
            last_role_sync_at: entity.last_role_sync_at,
        })
    }
}

/// Parameters for upserting a user during authentication or sync.
///
/// Creates new users or updates existing user information. The optional `is_admin`
/// field preserves existing admin status when None, allowing updates to name without
/// modifying permissions.
#[derive(Debug, Clone)]
pub struct UpsertUserParam {
    /// Discord ID of the user
    pub discord_id: u64,
    /// Display name of the user.
    pub name: String,
    /// Optional admin status (None preserves existing admin status, Some updates it).
    pub is_admin: Option<bool>,
}

/// Paginated collection of users with metadata.
///
/// Contains a page of users along with pagination metadata for building
/// paginated user management interfaces. Includes total counts and page information
/// for navigation controls.
#[derive(Debug, Clone, PartialEq)]
pub struct PaginatedUsers {
    /// Users for this page.
    pub users: Vec<User>,
    /// Total number of users across all pages.
    pub total: u64,
    /// Current page number (zero-indexed).
    pub page: u64,
    /// Number of users per page.
    pub per_page: u64,
    /// Total number of pages.
    pub total_pages: u64,
}

impl PaginatedUsers {
    /// Converts the paginated users domain model to a DTO for API responses.
    ///
    /// Converts each user in the collection to a DTO. If any user conversion fails,
    /// returns an error immediately without processing remaining users.
    ///
    /// # Returns
    /// - `Ok(PaginatedUsersDto)` - Successfully converted all users
    /// - `Err(String)` - Failed to parse discord_id for at least one user
    pub fn into_dto(self) -> PaginatedUsersDto {
        let users = self.users.into_iter().map(|u| u.into_dto()).collect();

        PaginatedUsersDto {
            users,
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}

/// Parameters for querying a user by Discord ID.
///
/// Used when fetching user information for a specific Discord user.
#[derive(Debug, Clone)]
pub struct GetUserParam {
    /// Discord ID of the user to retrieve.
    pub discord_id: u64,
}

/// Parameters for paginated user queries.
///
/// Specifies which page and how many users per page to retrieve.
#[derive(Debug, Clone)]
pub struct GetAllUsersParam {
    /// Zero-indexed page number.
    pub page: u64,
    /// Number of users to return per page.
    pub per_page: u64,
}

/// Parameters for setting user admin status.
///
/// Used to grant or revoke admin privileges for a specific user.
#[derive(Debug, Clone)]
pub struct SetAdminParam {
    /// Discord ID of the user to modify.
    pub discord_id: u64,
    /// Whether the user should have admin privileges.
    pub is_admin: bool,
}
