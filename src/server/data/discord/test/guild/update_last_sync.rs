use super::*;
use sea_orm::ActiveModelTrait;

/// Tests updating last_sync_at for existing guild.
///
/// Verifies that the last_sync_at timestamp is updated to
/// approximately the current time.
///
/// Expected: Ok with timestamp updated
#[tokio::test]
async fn updates_last_sync_timestamp() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild with old sync time
    let old_sync = Utc::now() - chrono::Duration::hours(2);
    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(old_sync),
    }
    .insert(db)
    .await?;

    let before_update = Utc::now();

    let repo = DiscordGuildRepository::new(db);
    let result = repo.update_last_sync(123456789).await;

    let after_update = Utc::now();

    assert!(result.is_ok());

    // Verify timestamp was updated
    let db_guild = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?
        .unwrap();

    let updated_time = db_guild.last_sync_at;

    // Verify timestamp is between before and after (with small buffer)
    assert!(updated_time >= old_sync);
    assert!(updated_time >= before_update - chrono::Duration::seconds(1));
    assert!(updated_time <= after_update + chrono::Duration::seconds(1));

    Ok(())
}

/// Tests updating last_sync_at for nonexistent guild.
///
/// Verifies that updating a nonexistent guild succeeds
/// without error (no-op operation).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn succeeds_for_nonexistent_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = DiscordGuildRepository::new(db);
    let result = repo.update_last_sync(999999999).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests updating last_sync_at doesn't affect other guilds.
///
/// Verifies that updating one guild's timestamp doesn't
/// change other guilds' timestamps.
///
/// Expected: Ok with only specified guild updated
#[tokio::test]
async fn updates_only_specified_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let old_sync = Utc::now() - chrono::Duration::hours(2);

    // Create two guilds
    let _guild1 = entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("111111111".to_string()),
        name: sea_orm::ActiveValue::Set("Guild 1".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(old_sync),
    }
    .insert(db)
    .await?;

    let guild2 = entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("222222222".to_string()),
        name: sea_orm::ActiveValue::Set("Guild 2".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(old_sync),
    }
    .insert(db)
    .await?;

    // Update only guild1
    let repo = DiscordGuildRepository::new(db);
    repo.update_last_sync(111111111).await?;

    // Verify guild1 was updated
    let db_guild1 = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("111111111"))
        .one(db)
        .await?
        .unwrap();
    assert!(db_guild1.last_sync_at > old_sync);

    // Verify guild2 was not updated
    let db_guild2 = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("222222222"))
        .one(db)
        .await?
        .unwrap();
    let diff = (db_guild2.last_sync_at - guild2.last_sync_at)
        .num_seconds()
        .abs();
    assert!(diff < 2, "Guild 2 timestamp should not have changed");

    Ok(())
}

/// Tests updating last_sync_at multiple times.
///
/// Verifies that the timestamp can be updated multiple times
/// and each update reflects a newer timestamp.
///
/// Expected: Ok with progressively newer timestamps
#[tokio::test]
async fn updates_multiple_times() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let old_sync = Utc::now() - chrono::Duration::hours(2);
    entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(None),
        last_sync_at: sea_orm::ActiveValue::Set(old_sync),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);

    // First update
    repo.update_last_sync(123456789).await?;
    let db_guild1 = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?
        .unwrap();
    let first_update = db_guild1.last_sync_at;

    // Wait a tiny bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Second update
    repo.update_last_sync(123456789).await?;
    let db_guild2 = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?
        .unwrap();
    let second_update = db_guild2.last_sync_at;

    // Both should be newer than original
    assert!(first_update > old_sync);
    assert!(second_update >= first_update);

    Ok(())
}

/// Tests updating last_sync_at doesn't change other fields.
///
/// Verifies that updating the timestamp doesn't modify
/// name or icon_hash fields.
///
/// Expected: Ok with only timestamp changed
#[tokio::test]
async fn preserves_other_fields() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let old_sync = Utc::now() - chrono::Duration::hours(2);
    let original = entity::discord_guild::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        name: sea_orm::ActiveValue::Set("Test Guild".to_string()),
        icon_hash: sea_orm::ActiveValue::Set(Some("abc123".to_string())),
        last_sync_at: sea_orm::ActiveValue::Set(old_sync),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    repo.update_last_sync(123456789).await?;

    // Verify other fields unchanged
    let db_guild = entity::prelude::DiscordGuild::find()
        .filter(entity::discord_guild::Column::GuildId.eq("123456789"))
        .one(db)
        .await?
        .unwrap();

    assert_eq!(db_guild.guild_id, original.guild_id);
    assert_eq!(db_guild.name, original.name);
    assert_eq!(db_guild.icon_hash, original.icon_hash);
    assert!(db_guild.last_sync_at > original.last_sync_at);

    Ok(())
}
