use super::*;

/// Tests updating a ping format field's name.
///
/// Verifies that the repository successfully updates the name of an
/// existing field while preserving other properties.
///
/// Expected: Ok with field name updated
#[tokio::test]
async fn updates_field_name() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field = factory::ping_format_field::create_ping_format_field(
        db,
        ping_format.id,
        "Original Name",
        1,
    )
    .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: "Updated Name".to_string(),
            priority: field.priority,
            default_value: field.default_value.clone(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.id, field.id);
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.priority, field.priority);
    assert_eq!(updated.default_value, field.default_value);

    // Verify in database
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_field.name, "Updated Name");

    Ok(())
}

/// Tests updating a field's priority.
///
/// Verifies that the repository successfully updates the priority
/// of an existing field.
///
/// Expected: Ok with field priority updated
#[tokio::test]
async fn updates_field_priority() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Test Field")
        .priority(1)
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: field.name.clone(),
            priority: 10,
            default_value: field.default_value.clone(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.priority, 10);

    // Verify in database
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_field.priority, 10);

    Ok(())
}

/// Tests updating a field's default_value from Some to None.
///
/// Verifies that the repository successfully clears the default_value
/// of an existing field.
///
/// Expected: Ok with default_value set to None
#[tokio::test]
async fn updates_default_value_to_none() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Test Field")
        .priority(1)
        .default_value(Some("Original Value".to_string()))
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: field.name.clone(),
            priority: field.priority,
            default_value: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert!(updated.default_value.is_none());

    // Verify in database
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?
        .unwrap();
    assert!(db_field.default_value.is_none());

    Ok(())
}

/// Tests updating a field's default_value from None to Some.
///
/// Verifies that the repository successfully sets a default_value
/// on a field that previously had none.
///
/// Expected: Ok with default_value set to Some
#[tokio::test]
async fn updates_default_value_to_some() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Test Field")
        .priority(1)
        .default_value(None)
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: field.name.clone(),
            priority: field.priority,
            default_value: Some("New Default".to_string()),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.default_value, Some("New Default".to_string()));

    // Verify in database
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_field.default_value, Some("New Default".to_string()));

    Ok(())
}

/// Tests updating all field properties at once.
///
/// Verifies that the repository successfully updates name, priority,
/// and default_value in a single operation.
///
/// Expected: Ok with all properties updated
#[tokio::test]
async fn updates_all_properties() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Original Name")
        .priority(1)
        .default_value(Some("Original Value".to_string()))
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: "New Name".to_string(),
            priority: 5,
            default_value: Some("New Value".to_string()),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.id, field.id);
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.priority, 5);
    assert_eq!(updated.default_value, Some("New Value".to_string()));

    // Verify in database
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_field.name, "New Name");
    assert_eq!(db_field.priority, 5);
    assert_eq!(db_field.default_value, Some("New Value".to_string()));

    Ok(())
}

/// Tests updating with nonexistent field ID.
///
/// Verifies that the repository returns a RecordNotFound error when
/// attempting to update a field that doesn't exist.
///
/// Expected: Err(DbErr::RecordNotFound)
#[tokio::test]
async fn fails_for_nonexistent_field() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: 999999,
            name: "Test".to_string(),
            priority: 1,
            default_value: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DbErr::RecordNotFound(_) => (),
        err => panic!("Expected RecordNotFound error, got: {:?}", err),
    }

    Ok(())
}

/// Tests that update doesn't change ping_format_id.
///
/// Verifies that updating a field preserves its ping_format_id
/// and doesn't allow changing which format it belongs to.
///
/// Expected: Ok with ping_format_id unchanged
#[tokio::test]
async fn preserves_ping_format_id() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Test Field", 1)
            .await?;

    let original_format_id = field.ping_format_id;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: "Updated Name".to_string(),
            priority: 2,
            default_value: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.ping_format_id, original_format_id);

    Ok(())
}

/// Tests updating priority to negative value.
///
/// Verifies that fields can be updated to have negative priority values.
///
/// Expected: Ok with negative priority
#[tokio::test]
async fn updates_priority_to_negative() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Test Field")
        .priority(5)
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field.id,
            name: field.name.clone(),
            priority: -10,
            default_value: field.default_value.clone(),
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.priority, -10);

    Ok(())
}

/// Tests updating multiple fields independently.
///
/// Verifies that updating one field doesn't affect other fields
/// for the same ping format.
///
/// Expected: Ok with only specified field updated
#[tokio::test]
async fn updates_single_field_without_affecting_others() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let field1 =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Field 1", 1)
            .await?;
    let field2 =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Field 2", 2)
            .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .update(UpdatePingFormatFieldParam {
            id: field1.id,
            name: "Updated Field 1".to_string(),
            priority: 10,
            default_value: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "Updated Field 1");
    assert_eq!(updated.priority, 10);

    // Verify field2 is unchanged
    let db_field2 = entity::prelude::PingFormatField::find_by_id(field2.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_field2.name, "Field 2");
    assert_eq!(db_field2.priority, 2);

    Ok(())
}
