use super::*;
use sea_orm::ActiveModelTrait;

/// Tests guild needs sync when never synced.
///
/// Verifies that a guild returns true for needs_sync when
/// it doesn't exist in the database.
///
/// Expected: Ok(true)
#[tokio::test]
async fn needs_sync_for_nonexistent_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = DiscordGuildRepository::new(db);
    let result = repo.needs_sync(123456789).await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

/// Tests guild needs sync when synced recently.
///
/// Verifies that a guild synced within 30 minutes returns false.
///
/// Expected: Ok(false)
#[tokio::test]
async fn no_sync_needed_for_recent_sync() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild synced 5 minutes ago
    let sync_time = Utc::now() - chrono::Duration::minutes(5);
    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(sync_time),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.needs_sync(123456789).await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests guild needs sync when synced over 30 minutes ago.
///
/// Verifies that a guild synced more than 30 minutes ago returns true.
///
/// Expected: Ok(true)
#[tokio::test]
async fn needs_sync_for_old_sync() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild synced 60 minutes ago
    let sync_time = Utc::now() - chrono::Duration::minutes(60);
    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(sync_time),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.needs_sync(123456789).await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

/// Tests guild needs sync at exactly 30 minute boundary.
///
/// Verifies behavior at the 30-minute threshold.
///
/// Expected: Ok(false) - at threshold shouldn't need sync
#[tokio::test]
async fn no_sync_needed_at_threshold() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild synced just under 30 minutes ago to avoid timing precision issues
    let sync_time = Utc::now() - chrono::Duration::minutes(29) - chrono::Duration::seconds(50);
    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(sync_time),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.needs_sync(123456789).await;

    assert!(result.is_ok());
    // Just under 30 minutes, should not need sync (< threshold means > returns false)
    assert!(!result.unwrap());

    Ok(())
}

/// Tests guild needs sync for very old sync.
///
/// Verifies that guilds synced days ago need syncing.
///
/// Expected: Ok(true)
#[tokio::test]
async fn needs_sync_for_very_old_sync() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild synced 7 days ago
    let sync_time = Utc::now() - chrono::Duration::days(7);
    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(sync_time),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.needs_sync(123456789).await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}
