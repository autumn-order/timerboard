use super::*;

/// Tests deleting all relationships for a user.
///
/// Verifies that the repository successfully deletes all guild role
/// relationships for a specific user across all guilds.
///
/// Expected: Ok with all user relationships deleted
#[tokio::test]
async fn deletes_all_user_relationships() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::create_user(db).await?;
    let guild = factory::create_guild(db).await?;
    let _role1 = factory::create_guild_role(db, &guild.guild_id, "111111111").await?;
    let _role2 = factory::create_guild_role(db, &guild.guild_id, "222222222").await?;
    let _role3 = factory::create_guild_role(db, &guild.guild_id, "333333333").await?;

    let user_id = user.discord_id.parse().unwrap();
    let _rel1 = factory::create_user_guild_role(db, user_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user_id, 222222222).await?;
    let _rel3 = factory::create_user_guild_role(db, user_id, 333333333).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete_by_user(user_id).await;

    assert!(result.is_ok());

    // Verify no relationships remain for this user
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 0);

    // Verify user still exists
    let db_user = entity::prelude::User::find()
        .filter(entity::user::Column::DiscordId.eq(&user.discord_id))
        .one(db)
        .await?;
    assert!(db_user.is_some());

    Ok(())
}

/// Tests deleting relationships for user with no relationships.
///
/// Verifies that the repository returns Ok when attempting to delete
/// relationships for a user that has none (no-op).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn succeeds_for_user_with_no_relationships() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::create_user(db).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete_by_user(user.discord_id.parse().unwrap()).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting relationships for nonexistent user.
///
/// Verifies that the repository returns Ok when attempting to delete
/// relationships for a user that doesn't exist (no-op).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn succeeds_for_nonexistent_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete_by_user(999999999).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting relationships doesn't affect other users.
///
/// Verifies that deleting all relationships for one user doesn't
/// affect relationships for other users with the same roles.
///
/// Expected: Ok with only specified user's relationships deleted
#[tokio::test]
async fn deletes_only_specified_user_relationships() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user1 = factory::create_user(db).await?;
    let user2 = factory::create_user(db).await?;
    let guild = factory::create_guild(db).await?;
    let _role1 = factory::create_guild_role(db, &guild.guild_id, "111111111").await?;
    let _role2 = factory::create_guild_role(db, &guild.guild_id, "222222222").await?;

    let user1_id = user1.discord_id.parse().unwrap();
    let user2_id = user2.discord_id.parse().unwrap();

    // Create relationships for both users
    let _rel1 = factory::create_user_guild_role(db, user1_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user1_id, 222222222).await?;
    let _rel3 = factory::create_user_guild_role(db, user2_id, 111111111).await?;
    let _rel4 = factory::create_user_guild_role(db, user2_id, 222222222).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete_by_user(user1_id).await;

    assert!(result.is_ok());

    // Verify user1's relationships are deleted
    let count1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user1.discord_id))
        .count(db)
        .await?;
    assert_eq!(count1, 0);

    // Verify user2's relationships still exist
    let count2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user2.discord_id))
        .count(db)
        .await?;
    assert_eq!(count2, 2);

    Ok(())
}

/// Tests deleting relationships across multiple guilds.
///
/// Verifies that delete_by_user removes all of a user's role relationships
/// even when they span multiple guilds.
///
/// Expected: Ok with all relationships deleted across all guilds
#[tokio::test]
async fn deletes_relationships_across_multiple_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::create_user(db).await?;
    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;
    let _role1 = factory::create_guild_role(db, &guild1.guild_id, "111111111").await?;
    let _role2 = factory::create_guild_role(db, &guild1.guild_id, "222222222").await?;
    let _role3 = factory::create_guild_role(db, &guild2.guild_id, "333333333").await?;
    let _role4 = factory::create_guild_role(db, &guild2.guild_id, "444444444").await?;

    let user_id = user.discord_id.parse().unwrap();
    let _rel1 = factory::create_user_guild_role(db, user_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user_id, 222222222).await?;
    let _rel3 = factory::create_user_guild_role(db, user_id, 333333333).await?;
    let _rel4 = factory::create_user_guild_role(db, user_id, 444444444).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete_by_user(user_id).await;

    assert!(result.is_ok());

    // Verify all relationships are deleted
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 0);

    // Verify roles still exist in both guilds
    let role_count = entity::prelude::DiscordGuildRole::find().count(db).await?;
    assert_eq!(role_count, 4);

    Ok(())
}

/// Tests delete_by_user is idempotent.
///
/// Verifies that calling delete_by_user multiple times for the same
/// user doesn't cause errors.
///
/// Expected: Ok on all delete calls
#[tokio::test]
async fn idempotent_delete() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::create_user(db).await?;
    let guild = factory::create_guild(db).await?;
    let _role = factory::create_guild_role(db, &guild.guild_id, "123456789").await?;
    let _relationship =
        factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 123456789).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let user_id = user.discord_id.parse().unwrap();

    // Delete first time
    let result1 = repo.delete_by_user(user_id).await;
    assert!(result1.is_ok());

    // Delete second time (already deleted)
    let result2 = repo.delete_by_user(user_id).await;
    assert!(result2.is_ok());

    // Delete third time
    let result3 = repo.delete_by_user(user_id).await;
    assert!(result3.is_ok());

    Ok(())
}

/// Tests deleting relationships for user with single relationship.
///
/// Verifies that delete_by_user works correctly when the user has
/// only one role relationship.
///
/// Expected: Ok with single relationship deleted
#[tokio::test]
async fn deletes_single_relationship() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::DiscordGuildRole)
        .with_table(entity::prelude::UserDiscordGuildRole)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::create_user(db).await?;
    let guild = factory::create_guild(db).await?;
    let _role = factory::create_guild_role(db, &guild.guild_id, "123456789").await?;
    let _relationship =
        factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 123456789).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete_by_user(user.discord_id.parse().unwrap()).await;

    assert!(result.is_ok());

    // Verify relationship is deleted
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 0);

    Ok(())
}
