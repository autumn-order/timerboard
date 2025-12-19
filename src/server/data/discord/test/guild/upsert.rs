use super::*;
use sea_orm::{ActiveModelTrait, PaginatorTrait};

/// Tests upserting a new Discord guild.
///
/// Verifies that the repository successfully creates a new guild record
/// with the specified guild_id, name, and icon_hash from a Serenity Guild object.
///
/// Expected: Ok with guild created
#[tokio::test]
async fn upserts_new_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = create_test_guild(123456789, "Test Guild", Some("abc123"));

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    if let Err(ref e) = result {
        eprintln!("Upsert error: {:?}", e);
    }
    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert_eq!(upserted.guild_id, 123456789);
    assert_eq!(upserted.name, "Test Guild");
    assert_eq!(
        upserted.icon_hash,
        Some("abc12300000000000000000000000000".to_string())
    );

    // Verify guild exists in database
    let db_guild = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?;
    assert!(db_guild.is_some());

    Ok(())
}

/// Tests upserting updates existing guild.
///
/// Verifies that when a guild with the same guild_id already exists,
/// the upsert operation updates the name and icon_hash fields
/// rather than creating a duplicate.
///
/// Expected: Ok with guild updated
#[tokio::test]
async fn updates_existing_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create initial guild
    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Old Name")
        .icon_hash(Some("old_icon".to_string()))
        .build()
        .await?;

    // Upsert with new values (use valid hex string)
    let guild = create_test_guild(123456789, "New Name", Some("abcdef12"));

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert_eq!(upserted.name, "New Name");
    assert_eq!(
        upserted.icon_hash,
        Some("abcdef12000000000000000000000000".to_string())
    );

    // Verify only one guild exists
    let count = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .count(db)
        .await?;
    assert_eq!(count, 1);

    Ok(())
}

/// Tests upserting guild with None icon_hash.
///
/// Verifies that guilds without an icon (icon_hash is None)
/// are properly stored.
///
/// Expected: Ok with None icon_hash
#[tokio::test]
async fn upserts_guild_with_none_icon() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = create_test_guild(123456789, "No Icon Guild", None);

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert!(upserted.icon_hash.is_none());

    Ok(())
}

/// Tests upserting guild updates icon from Some to None.
///
/// Verifies that when a guild previously had an icon but now doesn't,
/// the icon_hash is properly updated to None.
///
/// Expected: Ok with icon_hash updated to None
#[tokio::test]
async fn updates_icon_from_some_to_none() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild with icon
    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Test Guild")
        .icon_hash(Some("old_icon".to_string()))
        .build()
        .await?;

    // Upsert with no icon
    let guild = create_test_guild(123456789, "Test Guild", None);

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert!(upserted.icon_hash.is_none());

    // Verify in database
    let db_guild = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?
        .unwrap();
    assert!(db_guild.icon_hash.is_none());

    Ok(())
}

/// Tests upserting guild updates icon from None to Some.
///
/// Verifies that when a guild previously had no icon but now has one,
/// the icon_hash is properly updated to Some.
///
/// Expected: Ok with icon_hash updated to Some
#[tokio::test]
async fn updates_icon_from_none_to_some() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild without icon
    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Test Guild")
        .icon_hash(None)
        .build()
        .await?;

    // Upsert with icon (use valid hex string)
    let guild = create_test_guild(123456789, "Test Guild", Some("abcdef12"));

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert_eq!(
        upserted.icon_hash,
        Some("abcdef12000000000000000000000000".to_string())
    );

    Ok(())
}

/// Tests upserting multiple different guilds.
///
/// Verifies that multiple guilds with different guild_ids can be upserted
/// without conflicts.
///
/// Expected: Ok with multiple guilds created
#[tokio::test]
async fn upserts_multiple_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = create_test_guild(111111111, "Guild 1", Some("icon1"));
    let guild2 = create_test_guild(222222222, "Guild 2", Some("icon2"));
    let guild3 = create_test_guild(333333333, "Guild 3", None);

    let repo = DiscordGuildRepository::new(db);
    repo.upsert(guild1).await?;
    repo.upsert(guild2).await?;
    repo.upsert(guild3).await?;

    // Verify all guilds exist
    let count = entity::prelude::DiscordGuild::find().count(db).await?;
    assert_eq!(count, 3);

    Ok(())
}

/// Tests upserting guild with special characters in name.
///
/// Verifies that guild names with special characters, emojis, and
/// Unicode are properly stored.
///
/// Expected: Ok with special characters preserved
#[tokio::test]
async fn upserts_guild_with_special_characters() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = create_test_guild(
        123456789,
        "Guild 🎮 with émojis & spëcial ⭐ chars!",
        Some("icon"),
    );

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert_eq!(upserted.name, "Guild 🎮 with émojis & spëcial ⭐ chars!");

    Ok(())
}

/// Tests upserting preserves last_sync_at timestamp.
///
/// Verifies that upserting a guild doesn't modify the last_sync_at
/// timestamp, which should only be updated via update_last_sync().
///
/// Expected: Ok with last_sync_at unchanged
#[tokio::test]
async fn preserves_last_sync_at() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild with specific last_sync_at
    let original_time = Utc::now() - chrono::Duration::hours(2);

    // Manually insert with specific last_sync_at since factory doesn't support it
    let _original_guild = entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(original_time),
    }
    .insert(db)
    .await?;

    // Upsert the same guild with updated name
    let guild = create_test_guild(123456789, "Updated Name", Some("icon"));

    let repo = DiscordGuildRepository::new(db);
    repo.upsert(guild).await?;

    // Verify last_sync_at wasn't changed
    let db_guild = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?
        .unwrap();

    // Compare timestamps (allowing for small precision differences)
    let diff = (db_guild.last_sync_at - original_time).num_seconds().abs();
    assert!(diff < 2, "last_sync_at should not have changed");

    Ok(())
}

/// Tests upserting guild with very long name.
///
/// Verifies that guilds with long names (up to Discord's limit)
/// are properly stored.
///
/// Expected: Ok with full name preserved
#[tokio::test]
async fn upserts_guild_with_long_name() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let long_name = "A".repeat(100); // Discord max is 100 characters
    let guild = create_test_guild(123456789, &long_name, Some("icon"));

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert_eq!(upserted.name, long_name);
    assert_eq!(upserted.name.len(), 100);

    Ok(())
}

/// Tests upserting guild with long icon hash.
///
/// Verifies that icon hashes are properly stored.
///
/// Expected: Ok with icon hash preserved
#[tokio::test]
async fn upserts_guild_with_long_icon_hash() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let icon_hash = "a_".to_string() + &"0".repeat(32); // Animated icon prefix + 32 hex chars = 34 total
    let guild = create_test_guild(123456789, "Test Guild", Some(&icon_hash));

    let repo = DiscordGuildRepository::new(db);
    let result = repo.upsert(guild).await;

    assert!(result.is_ok());
    let upserted = result.unwrap();
    assert_eq!(
        upserted.icon_hash,
        Some("a_00000000000000000000000000000000".to_string())
    );

    Ok(())
}
