use chrono::{DateTime, Utc};

use crate::model::user::UserDto;

/// User with Discord identity, permissions, and sync metadata.
///
/// Tracks user's Discord ID, display name, admin privileges, and when their
/// guild memberships and role assignments were last synchronized.
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    /// Discord ID of the user (stored as String in database).
    pub discord_id: String,
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
    /// # Returns
    /// - `UserDto` - The converted user DTO with discord_id as u64
    pub fn into_dto(self) -> UserDto {
        UserDto {
            discord_id: self.discord_id.parse().unwrap_or(0),
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
    /// - `User` - The converted user domain model
    pub fn from_entity(entity: entity::user::Model) -> Self {
        Self {
            discord_id: entity.discord_id,
            name: entity.name,
            admin: entity.admin,
            last_guild_sync_at: entity.last_guild_sync_at,
            last_role_sync_at: entity.last_role_sync_at,
        }
    }
}

/// Parameters for upserting a user during authentication or sync.
///
/// Creates new users or updates existing user information. The optional `is_admin`
/// field preserves existing admin status when None.
#[derive(Debug, Clone)]
pub struct UpsertUserParam {
    /// Discord ID of the user.
    pub discord_id: String,
    /// Display name of the user.
    pub name: String,
    /// Optional admin status (None means don't update existing admin status).
    pub is_admin: Option<bool>,
}
