use super::*;

/// Tests creating a new channel fleet list record.
///
/// Verifies that the repository successfully creates a new channel fleet list
/// record when one doesn't exist for the channel.
///
/// Expected: Ok with new record created
#[tokio::test]
async fn creates_new_record() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let channel_id = 123456789u64;
    let message_id = 987654321u64;

    let repo = ChannelFleetListRepository::new(db);
    let result = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id,
        })
        .await;

    assert!(result.is_ok());
    let channel_fleet_list = result.unwrap();
    assert_eq!(channel_fleet_list.channel_id, channel_id);
    assert_eq!(channel_fleet_list.message_id, message_id);

    // Verify record was created in database
    let stored = entity::prelude::ChannelFleetList::find()
        .filter(entity::channel_fleet_list::Column::ChannelId.eq(channel_id.to_string()))
        .one(db)
        .await?;

    assert!(stored.is_some());
    let stored = stored.unwrap();
    assert_eq!(stored.channel_id, channel_id.to_string());
    assert_eq!(stored.message_id, message_id.to_string());

    Ok(())
}

/// Tests updating an existing channel fleet list record.
///
/// Verifies that the repository successfully updates an existing record's
/// message_id when upserting with the same channel_id.
///
/// Expected: Ok with record updated
#[tokio::test]
async fn updates_existing_record() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let channel_id = 123456789u64;
    let original_message_id = 111111111u64;
    let new_message_id = 222222222u64;

    let repo = ChannelFleetListRepository::new(db);

    // Create initial record
    let original = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: original_message_id,
        })
        .await?;

    // Update with new message_id
    let result = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: new_message_id,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.id, original.id); // Same record ID
    assert_eq!(updated.channel_id, channel_id);
    assert_eq!(updated.message_id, new_message_id);

    // Verify only one record exists for this channel
    let count = entity::prelude::ChannelFleetList::find()
        .filter(entity::channel_fleet_list::Column::ChannelId.eq(channel_id.to_string()))
        .count(db)
        .await?;

    assert_eq!(count, 1);

    Ok(())
}

/// Tests that upsert updates last_message_at timestamp.
///
/// Verifies that both creating and updating a record sets the last_message_at
/// timestamp to the current time.
///
/// Expected: Ok with timestamps set
#[tokio::test]
async fn updates_last_message_at_timestamp() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let channel_id = 123456789u64;
    let repo = ChannelFleetListRepository::new(db);

    // Create initial record
    let first = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: 111111111u64,
        })
        .await?;

    assert!(first.last_message_at > first.created_at - chrono::Duration::seconds(1));

    // Wait a moment to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Update record
    let second = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: 222222222u64,
        })
        .await?;

    assert!(second.last_message_at > first.last_message_at);

    Ok(())
}

/// Tests that upsert preserves created_at timestamp on update.
///
/// Verifies that when updating an existing record, the created_at timestamp
/// remains unchanged from the original creation.
///
/// Expected: Ok with created_at preserved
#[tokio::test]
async fn preserves_created_at_on_update() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let channel_id = 123456789u64;
    let repo = ChannelFleetListRepository::new(db);

    // Create initial record
    let original = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: 111111111u64,
        })
        .await?;

    // Wait a moment
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Update record
    let updated = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: 222222222u64,
        })
        .await?;

    assert_eq!(updated.created_at, original.created_at);

    Ok(())
}

/// Tests that upsert updates updated_at timestamp.
///
/// Verifies that both creating and updating a record sets the updated_at
/// timestamp to the current time.
///
/// Expected: Ok with updated_at timestamps set
#[tokio::test]
async fn updates_updated_at_timestamp() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let channel_id = 123456789u64;
    let repo = ChannelFleetListRepository::new(db);

    // Create initial record
    let first = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: 111111111u64,
        })
        .await?;

    assert!(first.updated_at > first.created_at - chrono::Duration::seconds(1));

    // Wait a moment to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Update record
    let second = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id,
            message_id: 222222222u64,
        })
        .await?;

    assert!(second.updated_at > first.updated_at);

    Ok(())
}

/// Tests upserting multiple different channels.
///
/// Verifies that upserting records for different channels creates separate
/// records that don't interfere with each other.
///
/// Expected: Ok with separate records for each channel
#[tokio::test]
async fn creates_separate_records_for_different_channels() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = ChannelFleetListRepository::new(db);

    // Create records for three different channels
    let channel1 = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id: 111111111u64,
            message_id: 111111111u64,
        })
        .await?;

    let channel2 = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id: 222222222u64,
            message_id: 222222222u64,
        })
        .await?;

    let channel3 = repo
        .upsert(UpsertChannelFleetListParam {
            channel_id: 333333333u64,
            message_id: 333333333u64,
        })
        .await?;

    // Verify all three records exist with different IDs
    assert_ne!(channel1.id, channel2.id);
    assert_ne!(channel1.id, channel3.id);
    assert_ne!(channel2.id, channel3.id);

    // Verify total count
    let count = entity::prelude::ChannelFleetList::find().count(db).await?;
    assert_eq!(count, 3);

    Ok(())
}
