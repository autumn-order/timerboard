use super::*;

/// Tests finding an existing user by Discord ID.
///
/// Verifies that the repository successfully retrieves a user record
/// when queried with a Discord ID that exists in the database.
///
/// Expected: Ok(Some(User)) with matching user data
#[tokio::test]
async fn finds_existing_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create a user
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "TestUser".to_string(),
        is_admin: Some(true),
    })
    .await?;

    // Find the user
    let result = repo.find_by_discord_id(123456789).await;

    assert!(result.is_ok());
    let user_opt = result.unwrap();
    assert!(user_opt.is_some());
    let user = user_opt.unwrap();
    assert_eq!(user.discord_id, "123456789");
    assert_eq!(user.name, "TestUser");
    assert!(user.admin);

    Ok(())
}

/// Tests querying for a non-existent user.
///
/// Verifies that the repository returns None when queried with a Discord ID
/// that does not exist in the database.
///
/// Expected: Ok(None)
#[tokio::test]
async fn returns_none_for_nonexistent_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo.find_by_discord_id(999999999).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    Ok(())
}
