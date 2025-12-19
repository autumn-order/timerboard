use super::*;
use chrono::Utc;
use std::thread::sleep;
use std::time::Duration;

/// Tests updating role sync timestamp for existing user.
///
/// Verifies that the repository successfully updates the last_role_sync_at
/// timestamp to the current time for an existing user.
///
/// Expected: Ok with timestamp updated to recent time
#[tokio::test]
async fn updates_timestamp_for_existing_user() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create a user
    let user = repo
        .upsert(UpsertUserParam {
            discord_id: "123456789".to_string(),
            name: "TestUser".to_string(),
            is_admin: None,
        })
        .await?;

    let original_timestamp = user.last_role_sync_at;

    // Wait a moment to ensure timestamp difference
    sleep(Duration::from_millis(10));

    // Update timestamp
    let result = repo.update_role_sync_timestamp(123456789).await;

    assert!(result.is_ok());

    // Verify timestamp was updated
    let updated_user = repo.find_by_discord_id(123456789).await?.unwrap();
    assert!(updated_user.last_role_sync_at > original_timestamp);
    assert!(updated_user.last_role_sync_at <= Utc::now());

    Ok(())
}

/// Tests updating timestamp for non-existent user.
///
/// Verifies that the repository handles updating a non-existent user
/// gracefully without returning an error (no-op behavior).
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
    let result = repo.update_role_sync_timestamp(999999999).await;

    assert!(result.is_ok());

    Ok(())
}
