use super::*;
use sea_orm::ActiveModelTrait;

/// Tests finding guild by guild_id when guild exists.
///
/// Verifies that the repository returns the guild when it exists
/// in the database.
///
/// Expected: Ok(Some(guild))
#[tokio::test]
async fn finds_existing_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let _created = factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Test Guild")
        .icon_hash(Some("abc123".to_string()))
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.find_by_guild_id(123456789).await;

    assert!(result.is_ok());
    let guild = result.unwrap();
    assert!(guild.is_some());

    let guild = guild.unwrap();
    assert_eq!(guild.guild_id, 123456789);
    assert_eq!(guild.name, "Test Guild");
    assert_eq!(guild.icon_hash, Some("abc123".to_string()));

    Ok(())
}

/// Tests finding guild by guild_id when guild doesn't exist.
///
/// Verifies that the repository returns None when the guild
/// is not found in the database.
///
/// Expected: Ok(None)
#[tokio::test]
async fn returns_none_for_nonexistent_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = DiscordGuildRepository::new(db);
    let result = repo.find_by_guild_id(999999999).await;

    assert!(result.is_ok());
    let guild = result.unwrap();
    assert!(guild.is_none());

    Ok(())
}

/// Tests finding guild returns complete data.
///
/// Verifies that all guild properties are correctly returned
/// including guild_id, name, icon_hash, and last_sync_at.
///
/// Expected: Ok(Some(guild)) with complete data
#[tokio::test]
async fn returns_complete_guild_data() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let sync_time = Utc::now() - chrono::Duration::minutes(10);

    // Manually insert with specific last_sync_at since factory doesn't support it
    let created = entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Complete Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(Some("icon_hash".to_string())),
        last_sync_at: sea_orm::ActiveValue::Set(sync_time),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.find_by_guild_id(123456789).await;

    assert!(result.is_ok());
    let guild = result.unwrap().unwrap();

    assert_eq!(guild.guild_id, 123456789);
    assert_eq!(guild.name, "Complete Guild");
    assert_eq!(guild.icon_hash, Some("icon_hash".to_string()));
    assert_eq!(guild.last_sync_at, created.last_sync_at);

    Ok(())
}

/// Tests finding guild with None icon_hash.
///
/// Verifies that guilds without icons are correctly retrieved.
///
/// Expected: Ok(Some(guild)) with None icon_hash
#[tokio::test]
async fn finds_guild_with_none_icon() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("No Icon Guild")
        .icon_hash(None)
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.find_by_guild_id(123456789).await;

    assert!(result.is_ok());
    let guild = result.unwrap().unwrap();
    assert!(guild.icon_hash.is_none());

    Ok(())
}

/// Tests finding specific guild among multiple guilds.
///
/// Verifies that the correct guild is returned when multiple
/// guilds exist in the database.
///
/// Expected: Ok(Some(guild)) with correct guild
#[tokio::test]
async fn finds_correct_guild_among_multiple() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create multiple guilds
    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("111111111")
        .name("Guild 1")
        .build()
        .await?;

    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("222222222")
        .name("Guild 2")
        .build()
        .await?;

    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("333333333")
        .name("Guild 3")
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.find_by_guild_id(222222222).await;

    assert!(result.is_ok());
    let guild = result.unwrap().unwrap();
    assert_eq!(guild.guild_id, 222222222);
    assert_eq!(guild.name, "Guild 2");

    Ok(())
}

/// Tests finding guild with special characters in name.
///
/// Verifies that guilds with special characters are correctly retrieved.
///
/// Expected: Ok(Some(guild)) with special characters preserved
#[tokio::test]
async fn finds_guild_with_special_characters() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("123456789")
        .name("Guild 🎮 with émojis!")
        .build()
        .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.find_by_guild_id(123456789).await;

    assert!(result.is_ok());
    let guild = result.unwrap().unwrap();
    assert_eq!(guild.name, "Guild 🎮 with émojis!");

    Ok(())
}
