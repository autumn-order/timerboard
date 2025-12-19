use super::*;

/// Tests detecting when admin users exist.
///
/// Verifies that the repository correctly returns true when at least one
/// admin user exists in the database.
///
/// Expected: Ok(true)
#[tokio::test]
async fn returns_true_when_admin_exists() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create an admin user
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "AdminUser".to_string(),
        is_admin: Some(true),
    })
    .await?;

    let result = repo.admin_exists().await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

/// Tests detecting when no admin users exist.
///
/// Verifies that the repository correctly returns false when no admin users
/// exist in the database (first-time setup scenario).
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_when_no_admins() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo.admin_exists().await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests detecting when only non-admin users exist.
///
/// Verifies that the repository correctly returns false when users exist
/// but none have admin privileges.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_with_only_regular_users() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create regular users
    repo.upsert(UpsertUserParam {
        discord_id: "111111111".to_string(),
        name: "User1".to_string(),
        is_admin: None,
    })
    .await?;

    repo.upsert(UpsertUserParam {
        discord_id: "222222222".to_string(),
        name: "User2".to_string(),
        is_admin: Some(false),
    })
    .await?;

    let result = repo.admin_exists().await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}
