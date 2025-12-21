use super::*;

/// Tests retrieving a channel fleet list by channel ID.
///
/// Verifies that the repository successfully retrieves a channel fleet list
/// record by its channel ID.
///
/// Expected: Ok(Some(channel_fleet_list))
#[tokio::test]
async fn returns_channel_fleet_list() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let channel_id = 123456789u64;
    let message_id = 987654321u64;

    // Create a channel fleet list record
    let repo = ChannelFleetListRepository::new(db);
    repo.upsert(UpsertChannelFleetListParam {
        channel_id,
        message_id,
    })
    .await?;

    // Retrieve the record
    let result = repo.get_by_channel_id(channel_id).await;

    assert!(result.is_ok());
    let channel_fleet_list = result.unwrap();
    assert!(channel_fleet_list.is_some());
    let channel_fleet_list = channel_fleet_list.unwrap();
    assert_eq!(channel_fleet_list.channel_id, channel_id);
    assert_eq!(channel_fleet_list.message_id, message_id);

    Ok(())
}

/// Tests retrieving non-existent channel fleet list.
///
/// Verifies that the repository returns None when querying for a channel ID
/// that doesn't exist in the database.
///
/// Expected: Ok(None)
#[tokio::test]
async fn returns_none_for_nonexistent_channel() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = ChannelFleetListRepository::new(db);
    let result = repo.get_by_channel_id(999999999u64).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    Ok(())
}

/// Tests channel fleet lists are isolated per channel.
///
/// Verifies that retrieving a channel fleet list for one channel does not
/// return records from other channels.
///
/// Expected: Ok with only the record for the queried channel
#[tokio::test]
async fn returns_only_record_for_specified_channel() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::ChannelFleetList)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = ChannelFleetListRepository::new(db);

    // Create records for two different channels
    let channel1_id = 111111111u64;
    let channel2_id = 222222222u64;

    repo.upsert(UpsertChannelFleetListParam {
        channel_id: channel1_id,
        message_id: 111111111u64,
    })
    .await?;

    repo.upsert(UpsertChannelFleetListParam {
        channel_id: channel2_id,
        message_id: 222222222u64,
    })
    .await?;

    // Get record for channel1
    let result = repo.get_by_channel_id(channel1_id).await;

    assert!(result.is_ok());
    let channel_fleet_list = result.unwrap().unwrap();
    assert_eq!(channel_fleet_list.channel_id, channel1_id);
    assert_eq!(channel_fleet_list.message_id, 111111111u64);

    Ok(())
}
