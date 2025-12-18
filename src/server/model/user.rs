use chrono::{DateTime, Utc};

use crate::model::user::UserDto;

/// Represents a user with full data from the database.
///
/// Contains all user information including Discord ID, name, admin status,
/// and synchronization timestamps.
#[derive(Debug, Clone, PartialEq)]
pub struct UserParam {
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

impl UserParam {
    /// Converts the user param to a DTO for API responses.
    ///
    /// # Arguments
    /// - `self` - The user param to convert
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

    /// Converts an entity model to a user param.
    ///
    /// This conversion happens at the data layer boundary to ensure entity models
    /// never leak into service or controller layers.
    ///
    /// # Arguments
    /// - `entity` - The entity model from the database
    ///
    /// # Returns
    /// - `UserParam` - The converted user param
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

/// Parameters for upserting (insert or update) a user.
///
/// Used when creating a new user or updating an existing user's information
/// during Discord authentication or sync operations.
#[derive(Debug, Clone)]
pub struct UpsertUserParam {
    /// Discord ID of the user.
    pub discord_id: String,
    /// Display name of the user.
    pub name: String,
    /// Optional admin status (None means don't update existing admin status).
    pub is_admin: Option<bool>,
}
