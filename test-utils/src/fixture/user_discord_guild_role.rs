//! Fixture for user Discord guild role test data.
//!
//! Provides fixture methods for creating in-memory user-guild-role relationship data
//! without database insertion. Used for unit testing and mocking.

use entity::user_discord_guild_role;

/// Default test user Discord ID.
pub const DEFAULT_USER_ID: &str = "123456789";

/// Default test role Discord ID.
pub const DEFAULT_ROLE_ID: &str = "987654321";

/// Creates a user Discord guild role entity model with default values.
///
/// Returns an in-memory entity without database insertion.
/// User ID defaults to "123456789", role ID defaults to "987654321".
///
/// # Returns
/// - `user_discord_guild_role::Model` - In-memory entity
///
/// # Example
/// ```rust,ignore
/// let entity = fixture::user_discord_guild_role::entity();
/// assert_eq!(entity.user_id, "123456789");
/// ```
pub fn entity() -> user_discord_guild_role::Model {
    user_discord_guild_role::Model {
        user_id: DEFAULT_USER_ID.to_string(),
        role_id: DEFAULT_ROLE_ID.to_string(),
    }
}

/// Creates a customizable user Discord guild role entity builder.
///
/// Use this when you need to override default values.
///
/// # Returns
/// - `UserDiscordGuildRoleEntityBuilder` - Builder with default values
///
/// # Example
/// ```rust,ignore
/// let entity = fixture::user_discord_guild_role::entity_builder()
///     .user_id("111111111")
///     .role_id("222222222")
///     .build();
/// ```
pub fn entity_builder() -> UserDiscordGuildRoleEntityBuilder {
    UserDiscordGuildRoleEntityBuilder::default()
}

/// Builder for user Discord guild role entity models.
///
/// Creates customizable entity models without database insertion.
/// All fields have sensible defaults for testing.
pub struct UserDiscordGuildRoleEntityBuilder {
    user_id: String,
    role_id: String,
}

impl Default for UserDiscordGuildRoleEntityBuilder {
    fn default() -> Self {
        Self {
            user_id: DEFAULT_USER_ID.to_string(),
            role_id: DEFAULT_ROLE_ID.to_string(),
        }
    }
}

impl UserDiscordGuildRoleEntityBuilder {
    /// Sets the user ID.
    ///
    /// # Arguments
    /// - `user_id` - Discord user ID
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = user_id.into();
        self
    }

    /// Sets the role ID.
    ///
    /// # Arguments
    /// - `role_id` - Discord role ID
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn role_id(mut self, role_id: impl Into<String>) -> Self {
        self.role_id = role_id.into();
        self
    }

    /// Builds the entity model.
    ///
    /// # Returns
    /// - `user_discord_guild_role::Model` - In-memory entity with configured values
    pub fn build(self) -> user_discord_guild_role::Model {
        user_discord_guild_role::Model {
            user_id: self.user_id,
            role_id: self.role_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_entity_with_defaults() {
        let relationship = entity();

        assert_eq!(relationship.user_id, DEFAULT_USER_ID);
        assert_eq!(relationship.role_id, DEFAULT_ROLE_ID);
    }

    #[test]
    fn builder_creates_entity_with_defaults() {
        let relationship = entity_builder().build();

        assert_eq!(relationship.user_id, DEFAULT_USER_ID);
        assert_eq!(relationship.role_id, DEFAULT_ROLE_ID);
    }

    #[test]
    fn builder_creates_entity_with_custom_values() {
        let relationship = entity_builder()
            .user_id("111111111")
            .role_id("222222222")
            .build();

        assert_eq!(relationship.user_id, "111111111");
        assert_eq!(relationship.role_id, "222222222");
    }

    #[test]
    fn builder_allows_partial_customization() {
        let relationship = entity_builder().user_id("999999999").build();

        assert_eq!(relationship.user_id, "999999999");
        assert_eq!(relationship.role_id, DEFAULT_ROLE_ID);
    }
}
