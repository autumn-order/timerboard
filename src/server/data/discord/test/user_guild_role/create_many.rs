use super::*;

/// Tests creating multiple relationships for a user.
///
/// Verifies that the repository successfully creates multiple user-guild-role
/// relationships for a single user with multiple roles.
///
/// Expected: Ok with all relationships created
#[tokio::test]
async fn creates_multiple_relationships() -> Result<(), DbErr> {
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
    let result = repo
        .create_many(
            user.discord_id.parse().unwrap(),
            &[111111111, 222222222, 333333333],
        )
        .await;

    assert!(result.is_ok());
    let relationships = result.unwrap();
    assert_eq!(relationships.len(), 3);

    // Verify all relationships exist in database
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 3);

    Ok(())
}

/// Tests creating many with empty role list.
///
/// Verifies that calling create_many with an empty slice returns
/// an empty vector without errors.
///
/// Expected: Ok with empty Vec
#[tokio::test]
async fn creates_none_with_empty_list() -> Result<(), DbErr> {
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
        .create_many(user.discord_id.parse().unwrap(), &[])
        .await;

    assert!(result.is_ok());
    let relationships = result.unwrap();
    assert_eq!(relationships.len(), 0);

    Ok(())
}

/// Tests creating many with single role.
///
/// Verifies that create_many works correctly with a single role,
/// equivalent to calling create once.
///
/// Expected: Ok with single relationship created
#[tokio::test]
async fn creates_single_relationship() -> Result<(), DbErr> {
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
        .create_many(user.discord_id.parse().unwrap(), &[123456789])
        .await;

    assert!(result.is_ok());
    let relationships = result.unwrap();
    assert_eq!(relationships.len(), 1);
    assert_eq!(
        relationships[0].user_id,
        user.discord_id.parse::<u64>().unwrap()
    );
    assert_eq!(relationships[0].role_id, 123456789);

    Ok(())
}

/// Tests creating many skips existing relationships.
///
/// Verifies that create_many only creates new relationships and
/// silently skips roles the user already has, avoiding duplicate errors.
///
/// Expected: Ok with only new relationships created
#[tokio::test]
async fn skips_existing_relationships() -> Result<(), DbErr> {
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

    // Create first two relationships
    repo.create(user_id, 111111111).await?;
    repo.create(user_id, 222222222).await?;

    // Attempt to create all three (two already exist)
    let result = repo
        .create_many(user_id, &[111111111, 222222222, 333333333])
        .await;

    assert!(result.is_ok());
    let relationships = result.unwrap();
    // Should only return the newly created relationship
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].role_id, 333333333);

    // Verify all three relationships exist in database
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 3);

    Ok(())
}

/// Tests creating many with all existing relationships.
///
/// Verifies that create_many returns an empty vector when all
/// relationships already exist.
///
/// Expected: Ok with empty Vec
#[tokio::test]
async fn returns_empty_when_all_exist() -> Result<(), DbErr> {
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
    let user_id = user.discord_id.parse().unwrap();

    // Create both relationships
    repo.create(user_id, 111111111).await?;
    repo.create(user_id, 222222222).await?;

    // Attempt to create the same relationships again
    let result = repo.create_many(user_id, &[111111111, 222222222]).await;

    assert!(result.is_ok());
    let relationships = result.unwrap();
    assert_eq!(relationships.len(), 0);

    // Verify still only two relationships
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Tests creating many with nonexistent user fails.
///
/// Verifies that attempting to create relationships with a user
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
    let result = repo.create_many(999999999, &[123456789]).await;

    assert!(result.is_err());

    Ok(())
}

/// Tests creating many with some nonexistent roles fails.
///
/// Verifies that attempting to create relationships with roles
/// that don't exist results in a database foreign key error.
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
    let guild = factory::create_guild(db).await?;
    let _role1 = factory::create_guild_role(db, &guild.guild_id, "111111111").await?;

    let repo = UserDiscordGuildRoleRepository::new(db);
    let result = repo
        .create_many(
            user.discord_id.parse().unwrap(),
            &[111111111, 999999999], // Second role doesn't exist
        )
        .await;

    assert!(result.is_err());

    Ok(())
}

/// Tests creating many with duplicate role IDs.
///
/// Verifies that create_many handles duplicate role IDs in the input
/// slice correctly by only creating one relationship.
///
/// Expected: Ok with single relationship created
#[tokio::test]
async fn handles_duplicate_role_ids_in_input() -> Result<(), DbErr> {
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
        .create_many(
            user.discord_id.parse().unwrap(),
            &[123456789, 123456789, 123456789], // Same role three times
        )
        .await;

    assert!(result.is_ok());
    let relationships = result.unwrap();
    // First creates, subsequent ones are skipped as existing
    assert_eq!(relationships.len(), 1);

    // Verify only one relationship exists
    let count = entity::prelude::UserDiscordGuildRole::find()
        .filter(entity::user_discord_guild_role::Column::UserId.eq(&user.discord_id))
        .count(db)
        .await?;
    assert_eq!(count, 1);

    Ok(())
}
