//! Test factory for creating Serenity Role objects.
//!
//! This module provides factory functions for creating mock Serenity `Role` structs
//! for testing purposes. These factories create valid Role objects by deserializing
//! JSON, simulating what Discord's API would return.

use serenity::all::Role;

/// Creates a test Serenity Role with customizable fields.
///
/// Creates a Role object by deserializing JSON with the provided values.
/// All other fields are set to reasonable defaults (not hoisted, not managed,
/// not mentionable, with zero permissions).
///
/// # Arguments
/// - `role_id` - Discord role ID (snowflake)
/// - `name` - Role name
/// - `color` - Role color as a 32-bit integer (RGB)
/// - `position` - Role position in the hierarchy (higher = more important)
///
/// # Returns
/// - `Role` - A valid Serenity Role struct for testing
///
/// # Panics
/// - If the JSON cannot be deserialized into a Role (indicates invalid test data)
///
/// # Examples
///
/// ```rust,ignore
/// use test_utils::serenity::role::create_test_role;
///
/// // Create basic role
/// let role = create_test_role(123456789, "Admin", 0xFF0000, 10);
/// assert_eq!(role.name, "Admin");
/// assert_eq!(role.colour.0, 0xFF0000);
/// assert_eq!(role.position, 10);
///
/// // Create role with zero color (default/no color)
/// let role = create_test_role(987654321, "Member", 0, 1);
///
/// // Create role with special characters
/// let role = create_test_role(111111111, "VIP ⭐", 0xFFD700, 5);
/// ```
pub fn create_test_role(role_id: u64, name: &str, color: u32, position: i16) -> Role {
    serde_json::from_value(serde_json::json!({
        "id": role_id.to_string(),
        "name": name,
        "color": color,
        "hoist": false,
        "icon": null,
        "unicode_emoji": null,
        "position": position,
        "permissions": "0",
        "managed": false,
        "mentionable": false,
    }))
    .expect("Failed to create test role - invalid JSON structure")
}
