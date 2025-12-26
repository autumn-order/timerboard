//! Ping format field fixtures for creating in-memory test data.
//!
//! Provides fixture functions for creating ping format field entity models without database insertion.
//! These are useful for unit testing, mocking, and providing consistent default values.

use entity::ping_format_field;

/// Default test ping format field name.
pub const DEFAULT_NAME: &str = "Test Field";

/// Default ping format ID for fields.
pub const DEFAULT_PING_FORMAT_ID: i32 = 1;

/// Default priority for fields.
pub const DEFAULT_PRIORITY: i32 = 1;

/// Default value for fields (None).
pub const DEFAULT_VALUE_TYPE: &str = "text";

/// Creates a ping format field entity model with default values.
///
/// This function creates an in-memory ping format field entity without inserting into the database.
/// Use this for unit tests and mocking repository responses.
///
/// # Default Values
/// - id: `1`
/// - ping_format_id: `1`
/// - name: `"Test Field"`
/// - priority: `1`
/// - default_value: `None`
///
/// # Returns
/// - `ping_format_field::Model` - In-memory ping format field entity
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::fixture;
///
/// let field = fixture::ping_format_field::entity();
/// assert_eq!(field.name, "Test Field");
/// assert_eq!(field.priority, 1);
/// ```
pub fn entity() -> ping_format_field::Model {
    ping_format_field::Model {
        id: 1,
        ping_format_id: DEFAULT_PING_FORMAT_ID,
        name: DEFAULT_NAME.to_string(),
        priority: DEFAULT_PRIORITY,
        field_type: DEFAULT_VALUE_TYPE.to_string(),
    }
}

/// Creates a ping format field entity builder for customization.
///
/// Provides a builder pattern for creating ping format field entities with custom values
/// while keeping sensible defaults for unspecified fields.
///
/// # Returns
/// - `PingFormatFieldEntityBuilder` - Builder instance with default values
///
/// # Example
///
/// ```rust,ignore
/// use test_utils::fixture;
///
/// let field = fixture::ping_format_field::entity_builder()
///     .name("Location")
///     .priority(5)
///     .default_value(Some("Jita".to_string()))
///     .build();
/// ```
pub fn entity_builder() -> PingFormatFieldEntityBuilder {
    PingFormatFieldEntityBuilder::default()
}

/// Builder for creating customized ping format field entity models.
///
/// Provides a fluent interface for building ping format field entities with custom values.
/// All fields have sensible defaults that can be overridden.
pub struct PingFormatFieldEntityBuilder {
    id: i32,
    ping_format_id: i32,
    name: String,
    priority: i32,
    value_type: String,
}

impl Default for PingFormatFieldEntityBuilder {
    fn default() -> Self {
        Self {
            id: 1,
            ping_format_id: DEFAULT_PING_FORMAT_ID,
            name: DEFAULT_NAME.to_string(),
            priority: DEFAULT_PRIORITY,
            value_type: DEFAULT_VALUE_TYPE.to_string(),
        }
    }
}

impl PingFormatFieldEntityBuilder {
    /// Sets the field ID.
    ///
    /// # Arguments
    /// - `id` - Ping format field ID
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }

    /// Sets the ping format ID.
    ///
    /// # Arguments
    /// - `ping_format_id` - ID of the ping format this field belongs to
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn ping_format_id(mut self, ping_format_id: i32) -> Self {
        self.ping_format_id = ping_format_id;
        self
    }

    /// Sets the field name.
    ///
    /// # Arguments
    /// - `name` - Display name for the field
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the field priority (order).
    ///
    /// # Arguments
    /// - `priority` - Sort priority for the field
    ///
    /// # Returns
    /// - `Self` - Builder instance for method chaining
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Builds and returns the ping format field entity model.
    ///
    /// # Returns
    /// - `ping_format_field::Model` - In-memory ping format field entity with configured values
    pub fn build(self) -> ping_format_field::Model {
        ping_format_field::Model {
            id: self.id,
            ping_format_id: self.ping_format_id,
            name: self.name,
            priority: self.priority,
            field_type: self.value_type,
        }
    }
}
