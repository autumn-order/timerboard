use super::*;

/// Tests creating a new user.
///
/// Verifies that the user repository successfully creates a new user record
/// with the specified Discord ID, name, and admin status.
///
/// Expected: Ok with user created and admin status set to false
#[tokio::test]
async fn creates_new_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo
        .upsert(UpsertUserParam {
            discord_id: "123456789".to_string(),
            name: "TestUser".to_string(),
            is_admin: None,
        })
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.discord_id, "123456789");
    assert_eq!(user.name, "TestUser");
    assert!(!user.admin);

    Ok(())
}

/// Tests creating a new user with admin status.
///
/// Verifies that the user repository successfully creates a new user record
/// with admin privileges when is_admin is Some(true).
///
/// Expected: Ok with user created and admin status set to true
#[tokio::test]
async fn creates_new_admin_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo
        .upsert(UpsertUserParam {
            discord_id: "123456789".to_string(),
            name: "AdminUser".to_string(),
            is_admin: Some(true),
        })
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.discord_id, "123456789");
    assert_eq!(user.name, "AdminUser");
    assert!(user.admin);

    Ok(())
}

/// Tests updating an existing user's name without affecting admin status.
///
/// Verifies that when upserting with is_admin as None, the user's name is updated
/// but the admin status is preserved from the original record.
///
/// Expected: Ok with name updated and admin status preserved
#[tokio::test]
async fn updates_existing_user_name_preserves_admin() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create initial user with admin status
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "OriginalName".to_string(),
        is_admin: Some(true),
    })
    .await?;

    // Update name without changing admin status
    let result = repo
        .upsert(UpsertUserParam {
            discord_id: "123456789".to_string(),
            name: "UpdatedName".to_string(),
            is_admin: None,
        })
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "UpdatedName");
    assert!(user.admin); // Should still be admin

    Ok(())
}

/// Tests updating an existing user's admin status.
///
/// Verifies that when upserting with is_admin as Some(true), both the name
/// and admin status are updated in the database.
///
/// Expected: Ok with both name and admin status updated
#[tokio::test]
async fn updates_existing_user_with_admin_status() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create initial non-admin user
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "RegularUser".to_string(),
        is_admin: None,
    })
    .await?;

    // Update to admin
    let result = repo
        .upsert(UpsertUserParam {
            discord_id: "123456789".to_string(),
            name: "AdminUser".to_string(),
            is_admin: Some(true),
        })
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "AdminUser");
    assert!(user.admin);

    Ok(())
}

/// Tests revoking admin status on update.
///
/// Verifies that when upserting with is_admin as Some(false), the user's
/// admin privileges are removed.
///
/// Expected: Ok with admin status set to false
#[tokio::test]
async fn revokes_admin_status_on_update() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create initial admin user
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "AdminUser".to_string(),
        is_admin: Some(true),
    })
    .await?;

    // Revoke admin
    let result = repo
        .upsert(UpsertUserParam {
            discord_id: "123456789".to_string(),
            name: "RegularUser".to_string(),
            is_admin: Some(false),
        })
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "RegularUser");
    assert!(!user.admin);

    Ok(())
}
