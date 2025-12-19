use super::*;
use sea_orm::ActiveModelTrait;

/// Tests getting guilds for user with multiple guild memberships.
///
/// Verifies that all guilds the user is a member of are returned.
///
/// Expected: Ok with all user's guilds
#[tokio::test]
async fn gets_all_guilds_for_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildMember)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::discord_guild::create_guild(db).await?;
    let guild2 = factory::discord_guild::create_guild(db).await?;
    let guild3 = factory::discord_guild::create_guild(db).await?;

    // Create memberships for user in guild1 and guild2
    entity::discord_guild_member::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set(guild1.guild_id.clone()),
        user_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        username: sea_orm::ActiveValue::Set("User123".to_string()),
        nickname: sea_orm::ActiveValue::Set(None),
    }
    .insert(db)
    .await?;

    entity::discord_guild_member::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set(guild2.guild_id.clone()),
        user_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        username: sea_orm::ActiveValue::Set("User123".to_string()),
        nickname: sea_orm::ActiveValue::Set(None),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_guilds_for_user(123456789).await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 2);

    let guild_ids: Vec<String> = guilds.iter().map(|g| g.guild_id.to_string()).collect();
    assert!(guild_ids.contains(&guild1.guild_id));
    assert!(guild_ids.contains(&guild2.guild_id));
    assert!(!guild_ids.contains(&guild3.guild_id));

    Ok(())
}

/// Tests getting guilds for user with no memberships.
///
/// Verifies that an empty vector is returned when the user
/// is not a member of any guilds.
///
/// Expected: Ok with empty vector
#[tokio::test]
async fn returns_empty_for_user_with_no_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildMember)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guilds but no memberships
    factory::discord_guild::create_guild(db).await?;
    factory::discord_guild::create_guild(db).await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_guilds_for_user(999999999).await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 0);

    Ok(())
}

/// Tests getting guilds filters by user correctly.
///
/// Verifies that only the specified user's guilds are returned,
/// not other users' guilds.
///
/// Expected: Ok with only specified user's guilds
#[tokio::test]
async fn filters_by_user_id() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildMember)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::discord_guild::create_guild(db).await?;
    let guild2 = factory::discord_guild::create_guild(db).await?;

    // User 1 in guild1, User 2 in guild2
    entity::discord_guild_member::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set(guild1.guild_id.clone()),
        user_id: sea_orm::ActiveValue::Set("111111111".to_string()),
        username: sea_orm::ActiveValue::Set("User1".to_string()),
        nickname: sea_orm::ActiveValue::Set(None),
    }
    .insert(db)
    .await?;

    entity::discord_guild_member::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set(guild2.guild_id.clone()),
        user_id: sea_orm::ActiveValue::Set("222222222".to_string()),
        username: sea_orm::ActiveValue::Set("User2".to_string()),
        nickname: sea_orm::ActiveValue::Set(None),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_guilds_for_user(111111111).await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 1);
    assert_eq!(guilds[0].guild_id.to_string(), guild1.guild_id);

    Ok(())
}

/// Tests getting guilds for single guild membership.
///
/// Verifies correct behavior when user is only in one guild.
///
/// Expected: Ok with single guild
#[tokio::test]
async fn gets_single_guild_for_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildMember)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    entity::discord_guild_member::ActiveModel {
        guild_id: sea_orm::ActiveValue::Set(guild.guild_id.clone()),
        user_id: sea_orm::ActiveValue::Set("123456789".to_string()),
        username: sea_orm::ActiveValue::Set("User123".to_string()),
        nickname: sea_orm::ActiveValue::Set(None),
    }
    .insert(db)
    .await?;

    let repo = DiscordGuildRepository::new(db);
    let result = repo.get_guilds_for_user(123456789).await;

    assert!(result.is_ok());
    let guilds = result.unwrap();
    assert_eq!(guilds.len(), 1);
    assert_eq!(guilds[0].guild_id.to_string(), guild.guild_id);

    Ok(())
}
