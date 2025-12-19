use super::*;

/// Tests granting admin status to a user.
///
/// Verifies that the repository successfully updates a user's admin status
/// to true, granting them admin privileges.
///
/// Expected: Ok with user admin status set to true
#[tokio::test]
async fn grants_admin_status() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create regular user
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "RegularUser".to_string(),
        is_admin: None,
    })
    .await?;

    // Grant admin
    let result = repo.set_admin(123456789, true).await;

    assert!(result.is_ok());

    // Verify admin status
    let user = repo.find_by_discord_id(123456789).await?.unwrap();
    assert!(user.admin);

    Ok(())
}

/// Tests revoking admin status from a user.
///
/// Verifies that the repository successfully updates a user's admin status
/// to false, removing their admin privileges.
///
/// Expected: Ok with user admin status set to false
#[tokio::test]
async fn revokes_admin_status() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create admin user
    repo.upsert(UpsertUserParam {
        discord_id: "123456789".to_string(),
        name: "AdminUser".to_string(),
        is_admin: Some(true),
    })
    .await?;

    // Revoke admin
    let result = repo.set_admin(123456789, false).await;

    assert!(result.is_ok());

    // Verify admin status revoked
    let user = repo.find_by_discord_id(123456789).await?.unwrap();
    assert!(!user.admin);

    Ok(())
}

/// Tests setting admin status for non-existent user.
///
/// Verifies that the repository handles setting admin status for a non-existent
/// user gracefully without returning an error (no-op behavior).
///
/// Expected: Ok (no error even though user doesn't exist)
#[tokio::test]
async fn succeeds_for_nonexistent_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo.set_admin(999999999, true).await;

    assert!(result.is_ok());

    Ok(())
}
