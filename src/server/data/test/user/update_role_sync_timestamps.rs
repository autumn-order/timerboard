use super::*;
use std::thread::sleep;
use std::time::Duration;

/// Tests batch updating timestamps for multiple users.
///
/// Verifies that the repository successfully updates the last_role_sync_at
/// timestamp for all specified users in a single operation.
///
/// Expected: Ok with all timestamps updated
#[tokio::test]
async fn updates_timestamps_for_multiple_users() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create multiple users
    let user1 = repo
        .upsert(UpsertUserParam {
            discord_id: "111111111".to_string(),
            name: "User1".to_string(),
            is_admin: None,
        })
        .await?;

    let user2 = repo
        .upsert(UpsertUserParam {
            discord_id: "222222222".to_string(),
            name: "User2".to_string(),
            is_admin: None,
        })
        .await?;

    let original_timestamp1 = user1.last_role_sync_at;
    let original_timestamp2 = user2.last_role_sync_at;

    sleep(Duration::from_millis(10));

    // Batch update timestamps
    let result = repo
        .update_role_sync_timestamps(&[111111111, 222222222])
        .await;

    assert!(result.is_ok());

    // Verify both timestamps were updated
    let updated_user1 = repo.find_by_discord_id(111111111).await?.unwrap();
    let updated_user2 = repo.find_by_discord_id(222222222).await?.unwrap();

    assert!(updated_user1.last_role_sync_at > original_timestamp1);
    assert!(updated_user2.last_role_sync_at > original_timestamp2);

    Ok(())
}

/// Tests batch update with empty slice.
///
/// Verifies that the repository handles an empty user ID slice gracefully
/// by returning early without attempting a database operation.
///
/// Expected: Ok (no-op for empty slice)
#[tokio::test]
async fn succeeds_with_empty_slice() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo.update_role_sync_timestamps(&[]).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests batch update with mix of existing and non-existent users.
///
/// Verifies that the repository successfully updates timestamps for existing
/// users while gracefully handling non-existent user IDs without errors.
///
/// Expected: Ok with existing users updated, non-existent IDs ignored
#[tokio::test]
async fn updates_only_existing_users() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create one user
    let user = repo
        .upsert(UpsertUserParam {
            discord_id: "111111111".to_string(),
            name: "User1".to_string(),
            is_admin: None,
        })
        .await?;

    let original_timestamp = user.last_role_sync_at;

    sleep(Duration::from_millis(10));

    // Update with mix of existing and non-existent IDs
    let result = repo
        .update_role_sync_timestamps(&[111111111, 999999999])
        .await;

    assert!(result.is_ok());

    // Verify existing user was updated
    let updated_user = repo.find_by_discord_id(111111111).await?.unwrap();
    assert!(updated_user.last_role_sync_at > original_timestamp);

    Ok(())
}
