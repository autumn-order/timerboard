use super::*;

/// Tests deleting an existing user-guild-role relationship.
///
/// Verifies that the repository successfully deletes a specific relationship
/// between a user and a guild role.
///
/// Expected: Ok with relationship deleted
#[tokio::test]
async fn deletes_relationship() -> Result<(), DbErr> {
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
    let result = repo
        .delete(user.discord_id.parse().unwrap(), 123456789)
        .await;

    assert!(result.is_ok());

    // Verify relationship no longer exists
    let db_relationship = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("123456789"))
        .one(db)
        .await?;
    assert!(db_relationship.is_none());

    Ok(())
}

/// Tests deleting nonexistent relationship.
///
/// Verifies that the repository returns Ok when attempting to delete
/// a relationship that doesn't exist (idempotent operation).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn succeeds_for_nonexistent_relationship() -> Result<(), DbErr> {
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

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo
        .delete(user.discord_id.parse().unwrap(), 123456789)
        .await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting relationship doesn't affect other relationships.
///
/// Verifies that deleting one user's role relationship doesn't affect
/// other users with the same role or the same user's other roles.
///
/// Expected: Ok with only specified relationship deleted
#[tokio::test]
async fn deletes_only_specified_relationship() -> Result<(), DbErr> {
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

    // Create multiple relationships
    let _rel1 = factory::create_user_guild_role(db, user1_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user1_id, 222222222).await?;
    let _rel3 = factory::create_user_guild_role(db, user2_id, 111111111).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete(user1_id, 111111111).await;

    assert!(result.is_ok());

    // Verify user1's role1 relationship is deleted
    let db_rel1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user1.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("111111111"))
        .one(db)
        .await?;
    assert!(db_rel1.is_none());

    // Verify user1's role2 relationship still exists
    let db_rel2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user1.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("222222222"))
        .one(db)
        .await?;
    assert!(db_rel2.is_some());

    // Verify user2's role1 relationship still exists
    let db_rel3 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user2.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("111111111"))
        .one(db)
        .await?;
    assert!(db_rel3.is_some());

    Ok(())
}

/// Tests deleting relationship is idempotent.
///
/// Verifies that calling delete on the same relationship multiple times
/// doesn't cause errors.
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
    let result1 = repo.delete(user_id, 123456789).await;
    assert!(result1.is_ok());

    // Delete second time (already deleted)
    let result2 = repo.delete(user_id, 123456789).await;
    assert!(result2.is_ok());

    // Delete third time
    let result3 = repo.delete(user_id, 123456789).await;
    assert!(result3.is_ok());

    Ok(())
}

/// Tests deleting relationship with nonexistent user.
///
/// Verifies that attempting to delete a relationship for a user
/// that doesn't exist succeeds (no-op).
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

    let guild = factory::create_guild(db).await?;
    let _role = factory::create_guild_role(db, &guild.guild_id, "123456789").await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.delete(999999999, 123456789).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting relationship with nonexistent role.
///
/// Verifies that attempting to delete a relationship for a role
/// that doesn't exist succeeds (no-op).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn succeeds_for_nonexistent_role() -> Result<(), DbErr> {
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
    let result = repo
        .delete(user.discord_id.parse().unwrap(), 999999999)
        .await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting all of a user's roles individually.
///
/// Verifies that multiple roles can be deleted individually and
/// the user can exist without any role relationships.
///
/// Expected: Ok with all relationships deleted
#[tokio::test]
async fn deletes_all_user_roles_individually() -> Result<(), DbErr> {
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
    repo.delete(user_id, 111111111).await?;
    repo.delete(user_id, 222222222).await?;
    repo.delete(user_id, 333333333).await?;

    // Verify no relationships remain
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
