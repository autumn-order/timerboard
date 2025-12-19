//! Shared helper utilities for factory methods.
//!
//! This module provides common utilities used across all factory modules,
//! including ID generation and convenience methods for creating entities
//! with their dependencies.

use sea_orm::{DatabaseConnection, DbErr};

/// Counter for generating unique IDs in tests.
///
/// This atomic counter ensures each factory-created entity gets a unique
/// identifier to prevent collisions in tests.
static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Gets the next unique counter value for test data.
///
/// This function provides monotonically increasing values for use in
/// generating unique test identifiers across all factories.
///
/// # Returns
/// - `u64` - Next unique counter value
pub fn next_id() -> u64 {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// Creates a complete fleet hierarchy with all dependencies.
///
/// This is a convenience method that creates:
/// 1. User (as fleet commander)
/// 2. Discord Guild
/// 3. Ping Format
/// 4. Fleet Category
/// 5. Fleet
///
/// All entities are created with default values. Use the individual
/// factories if you need to customize specific entities.
///
/// # Arguments
/// - `db` - Database connection
///
/// # Returns
/// - `Ok((user, guild, ping_format, category, fleet))` - Tuple of all created entities
/// - `Err(DbErr)` - Database error during creation
pub async fn create_fleet_with_dependencies(
    db: &DatabaseConnection,
) -> Result<
    (
        entity::user::Model,
        entity::discord_guild::Model,
        entity::ping_format::Model,
        entity::fleet_category::Model,
        entity::fleet::Model,
    ),
    DbErr,
> {
    let user = crate::factory::user::create_user(db).await?;
    let guild = crate::factory::discord_guild::create_guild(db).await?;
    let ping_format = crate::factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category =
        crate::factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id)
            .await?;
    let fleet = crate::factory::fleet::create_fleet(db, category.id, &user.discord_id).await?;

    Ok((user, guild, ping_format, category, fleet))
}

/// Creates a fleet with all dependencies using a specific user as commander.
///
/// This creates the necessary guild, ping format, and category structures,
/// then creates a fleet with the provided user as the commander. Useful
/// when you need to test fleet operations for a specific user.
///
/// # Arguments
/// - `db` - Database connection
/// - `user` - User entity to use as fleet commander
///
/// # Returns
/// - `Ok((guild, ping_format, category, fleet))` - Tuple of created entities
/// - `Err(DbErr)` - Database error during creation
pub async fn create_fleet_for_user(
    db: &DatabaseConnection,
    user: &entity::user::Model,
) -> Result<
    (
        entity::discord_guild::Model,
        entity::ping_format::Model,
        entity::fleet_category::Model,
        entity::fleet::Model,
    ),
    DbErr,
> {
    let guild = crate::factory::discord_guild::create_guild(db).await?;
    let ping_format = crate::factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category =
        crate::factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id)
            .await?;
    let fleet = crate::factory::fleet::create_fleet(db, category.id, &user.discord_id).await?;

    Ok((guild, ping_format, category, fleet))
}
