use super::*;

/// Tests creating a new user-guild-role relationship.
///
/// Verifies that the repository successfully creates a relationship between
/// a user and a guild role.
///
/// Expected: Ok with relationship created
#[tokio::test]
async fn creates_relationship() -> Result<(), DbErr> {
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
        .create(user.discord_id.parse().unwrap(), 123456789)
        .await;

    assert!(result.is_ok());
    let relationship = result.unwrap();
    assert_eq!(
        relationship.user_id,
        user.discord_id.parse::<u64>().unwrap()
    );
    assert_eq!(relationship.role_id, 123456789);

    // Verify relationship exists in database
    let db_relationship = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("123456789"))
        .one(db)
        .await?;
    assert!(db_relationship.is_some());

    Ok(())
}

/// Tests creating multiple relationships for different users.
///
/// Verifies that the repository can create separate relationships
/// for different users with the same role.
///
/// Expected: Ok with all relationships created
#[tokio::test]
async fn creates_relationships_for_different_users() -> Result<(), DbErr> {
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
    let _role = factory::create_guild_role(db, &guild.guild_id, "123456789").await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result1 = repo
        .create(user1.discord_id.parse().unwrap(), 123456789)
        .await;
    let result2 = repo
        .create(user2.discord_id.parse().unwrap(), 123456789)
        .await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Verify both relationships exist
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::RoleId.eq("123456789"))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Tests creating multiple relationships for the same user.
///
/// Verifies that the repository can create separate relationships
/// for the same user with different roles.
///
/// Expected: Ok with all relationships created
#[tokio::test]
async fn creates_relationships_for_different_roles() -> Result<(), DbErr> {
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

    let repo = UserDiscordGuildRoleRepository::new(db);
    let user_id = user.discord_id.parse().unwrap();
    let result1 = repo.create(user_id, 111111111).await;
    let result2 = repo.create(user_id, 222222222).await;
    let result3 = repo.create(user_id, 333333333).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    // Verify all relationships exist
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 3);

    Ok(())
}

/// Tests creating duplicate relationship fails.
///
/// Verifies that attempting to create a duplicate user-guild-role
/// relationship results in a database error due to primary key constraint.
///
/// Expected: Err with database constraint violation
#[tokio::test]
async fn fails_for_duplicate_relationship() -> Result<(), DbErr> {
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
    let user_id = user.discord_id.parse().unwrap();

    // Create first relationship
    let result1 = repo.create(user_id, 123456789).await;
    assert!(result1.is_ok());

    // Attempt to create duplicate
    let result2 = repo.create(user_id, 123456789).await;
    assert!(result2.is_err());

    Ok(())
}

/// Tests creating relationship with nonexistent user fails.
///
/// Verifies that attempting to create a relationship with a user
/// that doesn't exist results in a database foreign key error.
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
    let result = repo.create(999999999, 123456789).await;

    assert!(result.is_err());

    Ok(())
}

/// Tests creating relationship with nonexistent role fails.
///
/// Verifies that attempting to create a relationship with a role
/// that doesn't exist results in a database foreign key error.
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
        .create(user.discord_id.parse().unwrap(), 999999999)
        .await;

    assert!(result.is_err());

    Ok(())
}
