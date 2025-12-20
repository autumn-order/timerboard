use super::*;

/// Tests syncing user roles with new roles.
///
/// Verifies that the repository successfully replaces all existing role
/// relationships with a new set of roles for the user.
///
/// Expected: Ok with all old relationships deleted and new ones created
#[tokio::test]
async fn syncs_user_roles() -> Result<(), DbErr> {
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
    let _role4 = factory::create_guild_role(db, &guild.guild_id, "444444444").await?;

    let user_id = user.discord_id.parse().unwrap();

    // Create initial relationships
    let _rel1 = factory::create_user_guild_role(db, user_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user_id, 222222222).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.sync_user_roles(user_id, &[333333333, 444444444]).await;

    assert!(result.is_ok());

    // Verify old relationships are deleted
    let old_rel1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("111111111"))
        .one(db)
        .await?;
    assert!(old_rel1.is_none());

    let old_rel2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("222222222"))
        .one(db)
        .await?;
    assert!(old_rel2.is_none());

    // Verify new relationships exist
    let new_rel1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("333333333"))
        .one(db)
        .await?;
    assert!(new_rel1.is_some());

    let new_rel2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("444444444"))
        .one(db)
        .await?;
    assert!(new_rel2.is_some());

    // Verify total count is correct
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Tests syncing user roles with empty list.
///
/// Verifies that syncing with an empty role list removes all existing
/// relationships for the user.
///
/// Expected: Ok with all relationships deleted
#[tokio::test]
async fn syncs_to_empty_role_list() -> Result<(), DbErr> {
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

    let user_id = user.discord_id.parse().unwrap();

    // Create initial relationships
    let _rel1 = factory::create_user_guild_role(db, user_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user_id, 222222222).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.sync_user_roles(user_id, &[]).await;

    assert!(result.is_ok());

    // Verify all relationships are deleted
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 0);

    Ok(())
}

/// Tests syncing user roles when user has no existing roles.
///
/// Verifies that syncing works correctly when the user starts with
/// no role relationships.
///
/// Expected: Ok with new relationships created
#[tokio::test]
async fn syncs_from_empty_role_list() -> Result<(), DbErr> {
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

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo
        .sync_user_roles(user.discord_id.parse().unwrap(), &[111111111, 222222222])
        .await;

    assert!(result.is_ok());

    // Verify new relationships exist
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Tests syncing with same roles (no-op).
///
/// Verifies that syncing with the same roles the user already has
/// results in the same state (delete then re-create).
///
/// Expected: Ok with same relationships maintained
#[tokio::test]
async fn syncs_with_same_roles() -> Result<(), DbErr> {
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

    let user_id = user.discord_id.parse().unwrap();

    // Create initial relationships
    let _rel1 = factory::create_user_guild_role(db, user_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user_id, 222222222).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.sync_user_roles(user_id, &[111111111, 222222222]).await;

    assert!(result.is_ok());

    // Verify relationships still exist
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    // Verify specific relationships exist
    let rel1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("111111111"))
        .one(db)
        .await?;
    assert!(rel1.is_some());

    let rel2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("222222222"))
        .one(db)
        .await?;
    assert!(rel2.is_some());

    Ok(())
}

/// Tests syncing doesn't affect other users.
///
/// Verifies that syncing one user's roles doesn't affect role
/// relationships for other users.
///
/// Expected: Ok with only specified user's relationships changed
#[tokio::test]
async fn syncs_only_specified_user() -> Result<(), DbErr> {
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
    let _role3 = factory::create_guild_role(db, &guild.guild_id, "333333333").await?;

    let user1_id = user1.discord_id.parse().unwrap();
    let user2_id = user2.discord_id.parse().unwrap();

    // Create initial relationships for both users
    let _rel1 = factory::create_user_guild_role(db, user1_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user2_id, 222222222).await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo.sync_user_roles(user1_id, &[333333333]).await;

    assert!(result.is_ok());

    // Verify user1's relationships are synced
    let count1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user1.discord_id))
        .count(db)
        .await?;
    assert_eq!(count1, 1);

    let rel1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user1.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("333333333"))
        .one(db)
        .await?;
    assert!(rel1.is_some());

    // Verify user2's relationships are unchanged
    let count2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user2.discord_id))
        .count(db)
        .await?;
    assert_eq!(count2, 1);

    let rel2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user2.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("222222222"))
        .one(db)
        .await?;
    assert!(rel2.is_some());

    Ok(())
}

/// Tests syncing with partial overlap.
///
/// Verifies that syncing correctly handles cases where some roles
/// are kept and some are added/removed.
///
/// Expected: Ok with correct final state
#[tokio::test]
async fn syncs_with_partial_overlap() -> Result<(), DbErr> {
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
    let _role4 = factory::create_guild_role(db, &guild.guild_id, "444444444").await?;

    let user_id = user.discord_id.parse().unwrap();

    // Create initial relationships (role1, role2, role3)
    let _rel1 = factory::create_user_guild_role(db, user_id, 111111111).await?;
    let _rel2 = factory::create_user_guild_role(db, user_id, 222222222).await?;
    let _rel3 = factory::create_user_guild_role(db, user_id, 333333333).await?;

    // Sync to (role2, role3, role4) - keep role2 and role3, remove role1, add role4
    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo
        .sync_user_roles(user_id, &[222222222, 333333333, 444444444])
        .await;

    assert!(result.is_ok());

    // Verify final state
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 3);

    // Verify role1 is removed
    let rel1 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("111111111"))
        .one(db)
        .await?;
    assert!(rel1.is_none());

    // Verify role2, role3, role4 exist
    let rel2 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("222222222"))
        .one(db)
        .await?;
    assert!(rel2.is_some());

    let rel3 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("333333333"))
        .one(db)
        .await?;
    assert!(rel3.is_some());

    let rel4 = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("444444444"))
        .one(db)
        .await?;
    assert!(rel4.is_some());

    Ok(())
}

/// Tests syncing with nonexistent user fails.
///
/// Verifies that attempting to sync roles for a user that doesn't
/// exist results in a database foreign key error.
///
/// Expected: Err with foreign key constraint violation
#[tokio::test]
async fn fails_for_nonexistent_user() -> Result<(), DbErr> {
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
    let result = repo.sync_user_roles(999999999, &[123456789]).await;

    assert!(result.is_err());

    Ok(())
}

/// Tests syncing with nonexistent role fails.
///
/// Verifies that attempting to sync with roles that don't exist
/// results in a database foreign key error.
///
/// Expected: Err with foreign key constraint violation
#[tokio::test]
async fn fails_for_nonexistent_role() -> Result<(), DbErr> {
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
        .sync_user_roles(user.discord_id.parse().unwrap(), &[999999999])
        .await;

    assert!(result.is_err());

    Ok(())
}

/// Tests syncing is idempotent.
///
/// Verifies that calling sync_user_roles multiple times with the same
/// roles results in the same final state.
///
/// Expected: Ok with same state after multiple syncs
#[tokio::test]
async fn idempotent_sync() -> Result<(), DbErr> {
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

    let user_id = user.discord_id.parse().unwrap();

    let repo = UserDiscordGuildRoleRepository::new(db);

    // Sync first time
    let result1 = repo.sync_user_roles(user_id, &[111111111, 222222222]).await;
    assert!(result1.is_ok());

    // Sync second time with same roles
    let result2 = repo.sync_user_roles(user_id, &[111111111, 222222222]).await;
    assert!(result2.is_ok());

    // Sync third time
    let result3 = repo.sync_user_roles(user_id, &[111111111, 222222222]).await;
    assert!(result3.is_ok());

    // Verify final state is correct
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Tests syncing with large number of roles.
///
/// Verifies that syncing works correctly with many roles (stress test).
///
/// Expected: Ok with all relationships created
#[tokio::test]
async fn syncs_many_roles() -> Result<(), DbErr> {
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

    // Create 50 roles
    let mut role_ids = Vec::new();
    for i in 1..=50 {
        let role_id = 1000000 + i;
        let _role = factory::create_guild_role(db, &guild.guild_id, &role_id.to_string()).await?;
        role_ids.push(role_id);
    }

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo
        .sync_user_roles(user.discord_id.parse().unwrap(), &role_ids)
        .await;

    assert!(result.is_ok());

    // Verify all relationships exist
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 50);

    Ok(())
}
