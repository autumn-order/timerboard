use super::*;

/// Tests getting all guilds when multiple guilds exist.
///
/// Verifies that the repository returns all guild records
/// in the database.
///
/// Expected: Ok with all guilds returned
#[tokio::test]
async fn gets_all_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create multiple guilds
    factory::discord_guild::create_guild(db).await?;
    factory::discord_guild::create_guild(db).await?;
    factory::discord_guild::create_guild(db).await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 3);

    Ok(())
}

/// Tests getting all guilds when no guilds exist.
///
/// Verifies that the repository returns an empty vector when
/// there are no guild records in the database.
///
/// Expected: Ok with empty vector
#[tokio::test]
async fn returns_empty_when_no_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 0);

    Ok(())
}

/// Tests getting all guilds returns complete guild data.
///
/// Verifies that the returned domain models include all guild properties
/// including guild_id, name, icon_hash, and last_sync_at.
///
/// Expected: Ok with complete guild data
#[tokio::test]
async fn returns_complete_guild_data() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let created = factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Test Guild")
        .icon_hash(Some("abc123".to_string()))
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 1);

    let guild = &guilds[0];
    assert_eq!(guild.guild_id, 123456789);
    assert_eq!(guild.name, "Test Guild");
    assert_eq!(guild.icon_hash, Some("abc123".to_string()));
    assert_eq!(guild.last_sync_at, created.last_sync_at);

    Ok(())
}

/// Tests getting all guilds with single guild.
///
/// Verifies that the method works correctly when exactly one
/// guild exists in the database.
///
/// Expected: Ok with single guild returned
#[tokio::test]
async fn gets_single_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    factory::discord_guild::create_guild(db).await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 1);

    Ok(())
}

/// Tests getting all guilds with various icon_hash values.
///
/// Verifies that guilds with Some and None icon_hash values
/// are all returned correctly.
///
/// Expected: Ok with all guilds returned
#[tokio::test]
async fn gets_guilds_with_various_icon_hashes() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guilds with different icon_hash values
    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("111111111")
        .name("Guild with icon")
        .icon_hash(Some("icon_hash".to_string()))
        .build()
        .await?;

    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("222222222")
        .name("Guild without icon")
        .icon_hash(None)
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 2);

    // Verify both icon types are present
    let with_icon = guilds.iter().find(|g| g.guild_id == 111111111).unwrap();
    assert!(with_icon.icon_hash.is_some());

    let without_icon = guilds.iter().find(|g| g.guild_id == 222222222).unwrap();
    assert!(without_icon.icon_hash.is_none());

    Ok(())
}

/// Tests getting all guilds with special characters in names.
///
/// Verifies that guild names with special characters, emojis, and
/// Unicode are properly retrieved.
///
/// Expected: Ok with special characters preserved
#[tokio::test]
async fn gets_guilds_with_special_characters() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Guild 🎮 with émojis & spëcial ⭐ chars!")
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 1);
    assert_eq!(guilds[0].name, "Guild 🎮 with émojis & spëcial ⭐ chars!");

    Ok(())
}

/// Tests getting all guilds with various last_sync_at timestamps.
///
/// Verifies that guilds with different sync timestamps are all
/// returned correctly.
///
/// Expected: Ok with all guilds returned
#[tokio::test]
async fn gets_guilds_with_various_sync_times() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let now = Utc::now();

    // Create guilds with different sync times using ActiveModel
    use sea_orm::ActiveModelTrait;

    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("111111111".to_string()),
        name: sea_orm::ActiveValue::Set("Recently synced".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(now - chrono::Duration::minutes(5)),
    }
    .insert(db)
    .await?;

    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("222222222".to_string()),
        name: sea_orm::ActiveValue::Set("Old sync".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(now - chrono::Duration::hours(2)),
    }
    .insert(db)
    .await?;

    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("333333333".to_string()),
        name: sea_orm::ActiveValue::Set("Very old sync".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(now - chrono::Duration::days(7)),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_all().await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 3);

    // Verify all guilds are returned regardless of sync time
    let guild_ids: Vec<u64> = guilds.iter().map(|g| g.guild_id).collect();
    assert!(guild_ids.contains(&111111111));
    assert!(guild_ids.contains(&222222222));
    assert!(guild_ids.contains(&333333333));

    Ok(())
}
