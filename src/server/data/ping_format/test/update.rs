use super::*;

/// Tests updating a ping format's name.
///
/// Verifies that the repository successfully updates the name of an existing
/// ping format and returns the updated format.
///
/// Expected: Ok with updated ping format
#[tokio::test]
async fn updates_ping_format_name() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .update(UpdatePingFormatParam {
            id: ping_format.id,
            name: "Updated Name".to_string(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.id, ping_format.id);
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.guild_id.to_string(), guild.guild_id);

    // Verify update persisted in database
    let db_format = entity::prelude::PingFormat::find_by_id(ping_format.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_format.name, "Updated Name");

    Ok(())
}

/// Tests updating multiple ping formats.
///
/// Verifies that the repository can update multiple ping formats independently
/// without affecting other formats.
///
/// Expected: Ok with each format updated independently
#[tokio::test]
async fn updates_multiple_formats_independently() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let format1 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format3 = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);

    // Update format1
    let updated1 = repo
        .update(UpdatePingFormatParam {
            id: format1.id,
            name: "Updated 1".to_string(),
        })
        .await?;

    // Update format3
    let updated3 = repo
        .update(UpdatePingFormatParam {
            id: format3.id,
            name: "Updated 3".to_string(),
        })
        .await?;

    assert_eq!(updated1.name, "Updated 1");
    assert_eq!(updated3.name, "Updated 3");

    // Verify format2 unchanged
    let db_format2 = entity::prelude::PingFormat::find_by_id(format2.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_format2.name, format2.name);

    Ok(())
}

/// Tests updating a ping format to empty name.
///
/// Verifies that the repository allows updating a ping format to have an empty name,
/// as name validation is handled at the service/controller layer.
///
/// Expected: Ok with empty name accepted
#[tokio::test]
async fn updates_format_to_empty_name() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .update(UpdatePingFormatParam {
            id: ping_format.id,
            name: "".to_string(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "");

    Ok(())
}

/// Tests updating a ping format to long name.
///
/// Verifies that the repository can handle updating ping formats to have long names,
/// ensuring no truncation or errors occur at the data layer.
///
/// Expected: Ok with full name preserved
#[tokio::test]
async fn updates_format_to_long_name() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;
    let long_name = "B".repeat(255);

    let repo = PingFormatRepository::new(db);
    let result = repo
        .update(UpdatePingFormatParam {
            id: ping_format.id,
            name: long_name.clone(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, long_name);
    assert_eq!(updated.name.len(), 255);

    Ok(())
}

/// Tests updating a ping format with same name.
///
/// Verifies that updating a ping format to the same name it already has
/// succeeds without errors.
///
/// Expected: Ok with name unchanged
#[tokio::test]
async fn updates_format_with_same_name() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
        .name("Original Name")
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .update(UpdatePingFormatParam {
            id: ping_format.id,
            name: "Original Name".to_string(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "Original Name");

    Ok(())
}

/// Tests updating a nonexistent ping format.
///
/// Verifies that attempting to update a ping format that doesn't exist
/// returns a RecordNotFound error.
///
/// Expected: Err with RecordNotFound
#[tokio::test]
async fn fails_for_nonexistent_format() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatRepository::new(db);
    let result = repo
        .update(UpdatePingFormatParam {
            id: 999999,
            name: "Updated Name".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(_) => {}
        e => panic!("Expected NotFound, got: {:?}", e),
    }

    Ok(())
}

/// Tests updating a deleted ping format.
///
/// Verifies that attempting to update a ping format that was previously deleted
/// returns a RecordNotFound error.
///
/// Expected: Err with RecordNotFound
#[tokio::test]
async fn fails_for_deleted_format() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;
    let format_id = ping_format.id;

    // Delete the format
    entity::prelude::PingFormat::delete_by_id(format_id)
        .exec(db)
        .await?;

    // Try to update the deleted format
    let repo = PingFormatRepository::new(db);
    let result = repo
        .update(UpdatePingFormatParam {
            id: format_id,
            name: "Updated Name".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(_) => {}
        e => panic!("Expected NotFound, got: {:?}", e),
    }

    Ok(())
}

/// Tests updating ping format doesn't change guild association.
///
/// Verifies that updating a ping format's name doesn't affect its guild_id
/// or other fields.
///
/// Expected: Ok with only name changed
#[tokio::test]
async fn preserves_guild_id_on_update() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;
    let original_guild_id = ping_format.guild_id.clone();

    let repo = PingFormatRepository::new(db);
    let updated = repo
        .update(UpdatePingFormatParam {
            id: ping_format.id,
            name: "New Name".to_string(),
        })
        .await?;

    assert_eq!(updated.guild_id.to_string(), original_guild_id);
    assert_eq!(updated.id, ping_format.id);

    Ok(())
}
