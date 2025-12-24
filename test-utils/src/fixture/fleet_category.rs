//! Fleet category fixtures for creating in-memory test data.
//!
//! Provides fixture functions for creating fleet category entity models without database insertion.
//! These are useful for unit testing, mocking, and providing consistent default values.

use entity::fleet_category;

/// Default test fleet category name.
pub const DEFAULT_NAME: &str = "Test Category";

/// Default test guild ID for fleet categories.
pub const DEFAULT_GUILD_ID: &str = "987654321";

/// Default ping format ID for fleet categories.
pub const DEFAULT_PING_FORMAT_ID: i32 = 1;

/// Default ping cooldown (None).
pub const DEFAULT_PING_COOLDOWN: Option<i32> = None;

/// Default ping reminder (None).
pub const DEFAULT_PING_REMINDER: Option<i32> = None;

/// Default max pre-ping (None).
pub const DEFAULT_MAX_PRE_PING: Option<i32> = None;

/// Creates a fleet category entity model with default values.
///
/// This function creates an in-memory fleet category entity without inserting into the database.
/// Use this for unit tests and mocking repository responses.
///
/// # Default Values
/// - id: `1`
/// - guild_id: `"987654321"`
/// - ping_format_id: `1`
/// - name: `"Test Category"`
/// - ping_cooldown: `None`
/// - ping_reminder: `None`
/// - max_pre_ping: `None`
///
/// # Returns
/// - `fleet_category::Model` - In-memory fleet category entity
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::fixture;
///
/// let category = fixture::fleet_category::entity();
/// assert_eq!(category.name, "Test Category");
/// assert!(category.ping_cooldown.is_none());
/// ```
pub fn entity() -> fleet_category::Model {
    fleet_category::Model {
        id: 1,
        guild_id: DEFAULT_GUILD_ID.to_string(),
        ping_format_id: DEFAULT_PING_FORMAT_ID,
        ping_group_id: None,
        name: DEFAULT_NAME.to_string(),
        ping_cooldown: DEFAULT_PING_COOLDOWN,
        ping_reminder: DEFAULT_PING_REMINDER,
        max_pre_ping: DEFAULT_MAX_PRE_PING,
    }
}

/// Creates a fleet category entity builder for customization.
///
/// Provides a builder pattern for creating fleet category entities with custom values
/// while keeping sensible defaults for unspecified fields.
///
/// # Returns
/// - `FleetCategoryEntityBuilder` - Builder instance with default values
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::fixture;
///
/// let category = fixture::fleet_category::entity_builder()
///     .name("Strategic Ops")
///     .ping_cooldown(Some(60))
///     .build();
/// ```
pub fn entity_builder() -> FleetCategoryEntityBuilder {
    FleetCategoryEntityBuilder::default()
}

/// Builder for creating customized fleet category entity models.
///
/// Provides a fluent interface for building fleet category entities with custom values.
/// All fields have sensible defaults that can be overridden.
pub struct FleetCategoryEntityBuilder {
    id: i32,
    guild_id: String,
    ping_format_id: i32,
    name: String,
    ping_cooldown: Option<i32>,
    ping_reminder: Option<i32>,
    max_pre_ping: Option<i32>,
}

impl Default for FleetCategoryEntityBuilder {
    fn default() -> Self {
        Self {
            id: 1,
            guild_id: DEFAULT_GUILD_ID.to_string(),
            ping_format_id: DEFAULT_PING_FORMAT_ID,
            name: DEFAULT_NAME.to_string(),
            ping_cooldown: DEFAULT_PING_COOLDOWN,
            ping_reminder: DEFAULT_PING_REMINDER,
            max_pre_ping: DEFAULT_MAX_PRE_PING,
        }
    }
}

impl FleetCategoryEntityBuilder {
    /// Sets the category ID.
    ///
    /// # Arguments
    /// - `id` - Fleet category ID
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }

    /// Sets the guild ID.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn guild_id(mut self, guild_id: impl Into<String>) -> Self {
        self.guild_id = guild_id.into();
        self
    }

    /// Sets the ping format ID.
    ///
    /// # Arguments
    /// - `ping_format_id` - Ping format ID
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn ping_format_id(mut self, ping_format_id: i32) -> Self {
        self.ping_format_id = ping_format_id;
        self
    }

    /// Sets the category name.
    ///
    /// # Arguments
    /// - `name` - Display name for the category
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the ping cooldown in minutes.
    ///
    /// # Arguments
    /// - `cooldown` - Cooldown period between pings
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn ping_cooldown(mut self, cooldown: Option<i32>) -> Self {
        self.ping_cooldown = cooldown;
        self
    }

    /// Sets the ping reminder time in minutes.
    ///
    /// # Arguments
    /// - `reminder` - Minutes before fleet time to send reminder
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn ping_reminder(mut self, reminder: Option<i32>) -> Self {
        self.ping_reminder = reminder;
        self
    }

    /// Sets the maximum pre-ping time in minutes.
    ///
    /// # Arguments
    /// - `max_pre_ping` - Maximum minutes before fleet time to allow pings
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn max_pre_ping(mut self, max_pre_ping: Option<i32>) -> Self {
        self.max_pre_ping = max_pre_ping;
        self
    }

    /// Builds and returns the fleet category entity model.
    ///
    /// # Returns
    /// - `fleet_category::Model` - In-memory fleet category entity with configured values
    pub fn build(self) -> fleet_category::Model {
        fleet_category::Model {
            id: self.id,
            guild_id: self.guild_id,
            ping_format_id: self.ping_format_id,
            ping_group_id: None,
            name: self.name,
            ping_cooldown: self.ping_cooldown,
            ping_reminder: self.ping_reminder,
            max_pre_ping: self.max_pre_ping,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_entity_with_defaults() {
        let category = entity();

        assert_eq!(category.id, 1);
        assert_eq!(category.guild_id, DEFAULT_GUILD_ID);
        assert_eq!(category.ping_format_id, DEFAULT_PING_FORMAT_ID);
        assert_eq!(category.name, DEFAULT_NAME);
        assert_eq!(category.ping_cooldown, DEFAULT_PING_COOLDOWN);
        assert_eq!(category.ping_reminder, DEFAULT_PING_REMINDER);
        assert_eq!(category.max_pre_ping, DEFAULT_MAX_PRE_PING);
    }

    #[test]
    fn builder_creates_entity_with_defaults() {
        let category = entity_builder().build();

        assert_eq!(category.name, DEFAULT_NAME);
        assert!(category.ping_cooldown.is_none());
        assert!(category.ping_reminder.is_none());
        assert!(category.max_pre_ping.is_none());
    }

    #[test]
    fn builder_creates_entity_with_custom_values() {
        let category = entity_builder()
            .id(5)
            .guild_id("111222333")
            .ping_format_id(10)
            .name("Strategic Ops")
            .ping_cooldown(Some(60))
            .ping_reminder(Some(30))
            .max_pre_ping(Some(180))
            .build();

        assert_eq!(category.id, 5);
        assert_eq!(category.guild_id, "111222333");
        assert_eq!(category.ping_format_id, 10);
        assert_eq!(category.name, "Strategic Ops");
        assert_eq!(category.ping_cooldown, Some(60));
        assert_eq!(category.ping_reminder, Some(30));
        assert_eq!(category.max_pre_ping, Some(180));
    }

    #[test]
    fn builder_allows_partial_customization() {
        let category = entity_builder()
            .name("Partial Category")
            .ping_cooldown(Some(120))
            .build();

        assert_eq!(category.id, 1);
        assert_eq!(category.guild_id, DEFAULT_GUILD_ID);
        assert_eq!(category.name, "Partial Category");
        assert_eq!(category.ping_cooldown, Some(120));
        assert!(category.ping_reminder.is_none());
    }
}
