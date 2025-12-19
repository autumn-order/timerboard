//! Test factory for creating Serenity Guild objects.
//!
//! This module provides factory functions for creating mock Serenity `Guild` structs
//! for testing purposes. These factories create valid Guild objects by deserializing
//! JSON, simulating what Discord's API would return.

use serenity::all::Guild;

/// Creates a test Serenity Guild with customizable fields.
///
/// Creates a Guild object by deserializing JSON with the provided values.
/// The icon hash is automatically padded to 32 characters (Discord's icon hash format)
/// if it's shorter. All other fields are set to reasonable defaults.
///
/// # Arguments
/// - `guild_id` - Discord guild ID (snowflake)
/// - `name` - Guild name
/// - `icon_hash` - Optional icon hash (will be padded to 32 characters if shorter)
///
/// # Returns
/// - `Guild` - A valid Serenity Guild struct for testing
///
/// # Panics
/// - If the JSON cannot be deserialized into a Guild (indicates invalid test data)
///
/// # Examples
///
/// ```rust,ignore
/// use test_utils::serenity::guild::create_test_guild;
///
/// // Create guild without icon
/// let guild = create_test_guild(123456789, "Test Guild", None);
///
/// // Create guild with icon (automatically padded to 32 chars)
/// let guild = create_test_guild(123456789, "Test Guild", Some("abc123"));
/// assert_eq!(guild.icon_hash.unwrap().to_string(), "abc12300000000000000000000000000");
///
/// // Create guild with animated icon (34 chars: "a_" + 32 hex chars)
/// let guild = create_test_guild(123456789, "Test Guild", Some("a_abcdef1234567890abcdef1234567890"));
/// ```
pub fn create_test_guild(guild_id: u64, name: &str, icon_hash: Option<&str>) -> Guild {
    // Pad icon hash to be 32 characters if provided (Discord icon hash format)
    // Note: Animated icons should be 34 characters ("a_" prefix + 32 hex chars)
    let formatted_icon = icon_hash.map(|hash| {
        if hash.starts_with("a_") {
            // Animated icon - ensure it's 34 characters total
            if hash.len() < 34 {
                format!("{:0<34}", hash)
            } else {
                hash.to_string()
            }
        } else if hash.len() < 32 {
            // Normal icon - pad to 32 characters
            format!("{:0<32}", hash)
        } else {
            hash.to_string()
        }
    });

    serde_json::from_value(serde_json::json!({
        "id": guild_id.to_string(),
        "name": name,
        "icon": formatted_icon,
        "icon_hash": formatted_icon,
        "owner_id": "100000000000000000",
        "afk_timeout": 300,
        "verification_level": 0,
        "default_message_notifications": 0,
        "explicit_content_filter": 0,
        "roles": [],
        "emojis": [],
        "stickers": [],
        "features": [],
        "mfa_level": 0,
        "system_channel_flags": 0,
        "premium_tier": 0,
        "premium_subscription_count": 0,
        "premium_progress_bar_enabled": false,
        "preferred_locale": "en-US",
        "nsfw_level": 0,
        "joined_at": "2020-01-01T00:00:00.000000+00:00",
        "large": false,
        "member_count": 100,
        "voice_states": [],
        "channels": [],
        "threads": [],
        "presences": [],
        "max_presences": 25000,
        "max_members": 100000,
        "unavailable": false,
        "members": [],
        "stage_instances": [],
        "guild_scheduled_events": [],
    }))
    .expect("Failed to create test guild - invalid JSON structure")
}
