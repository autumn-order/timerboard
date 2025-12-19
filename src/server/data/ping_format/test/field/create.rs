use super::*;

/// Tests creating a new ping format field.
///
/// Verifies that the repository successfully creates a new field record
/// with the specified ping_format_id, name, priority, and default_value.
///
/// Expected: Ok with field created
#[tokio::test]
async fn creates_field() -> Result<(), DbErr> {
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

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Location".to_string(),
            priority: 1,
            default_value: Some("Default location".to_string()),
        })
        .await;

    assert!(result.is_ok());
    let field = result.unwrap();
    assert_eq!(field.ping_format_id, ping_format.id);
    assert_eq!(field.name, "Location");
    assert_eq!(field.priority, 1);
    assert_eq!(field.default_value, Some("Default location".to_string()));

    // Verify field exists in database
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?;
    assert!(db_field.is_some());

    Ok(())
}

/// Tests creating a field with None default_value.
///
/// Verifies that the repository successfully creates a field when
/// the default_value is None.
///
/// Expected: Ok with field created with None default_value
#[tokio::test]
async fn creates_field_with_none_default_value() -> Result<(), DbErr> {
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

    let repo = PingFormatFieldRepository::new(db);
    let result = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Custom Field".to_string(),
            priority: 2,
            default_value: None,
        })
        .await;

    assert!(result.is_ok());
    let field = result.unwrap();
    assert!(field.default_value.is_none());

    Ok(())
}

/// Tests creating multiple fields for the same ping format.
///
/// Verifies that multiple fields can be created for a single ping format
/// with different priorities and names.
///
/// Expected: Ok with multiple fields created
#[tokio::test]
async fn creates_multiple_fields_for_same_format() -> Result<(), DbErr> {
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

    let repo = PingFormatFieldRepository::new(db);
    let field1 = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Field 1".to_string(),
            priority: 1,
            default_value: None,
        })
        .await?;

    let field2 = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Field 2".to_string(),
            priority: 2,
            default_value: None,
        })
        .await?;

    assert_ne!(field1.id, field2.id);
    assert_eq!(field1.ping_format_id, field2.ping_format_id);

    // Verify both exist in database
    let count = entity::prelude::PingFormatField::find()
        .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
        .count(db)
        .await?;
    assert_eq!(count, 2);

    Ok(())
}

/// Tests foreign key constraint on ping_format_id.
///
/// Verifies that the repository returns an error when attempting to create
/// a field with a ping_format_id that doesn't exist in the database.
///
/// Expected: Err(DbErr) due to foreign key constraint violation
#[tokio::test]
async fn fails_for_nonexistent_ping_format() -> Result<(), DbErr> {
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
        .create(CreatePingFormatFieldParam {
            ping_format_id: 999999, // Non-existent ping format
            name: "Test Field".to_string(),
            priority: 1,
            default_value: None,
        })
        .await;

    assert!(result.is_err());

    Ok(())
}

/// Tests creating fields with different priorities.
///
/// Verifies that fields can be created with various priority values
/// including zero and negative numbers.
///
/// Expected: Ok with fields created with specified priorities
#[tokio::test]
async fn creates_fields_with_various_priorities() -> Result<(), DbErr> {
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

    let repo = PingFormatFieldRepository::new(db);

    let field_zero = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Zero Priority".to_string(),
            priority: 0,
            default_value: None,
        })
        .await?;
    assert_eq!(field_zero.priority, 0);

    let field_negative = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Negative Priority".to_string(),
            priority: -1,
            default_value: None,
        })
        .await?;
    assert_eq!(field_negative.priority, -1);

    let field_high = repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "High Priority".to_string(),
            priority: 100,
            default_value: None,
        })
        .await?;
    assert_eq!(field_high.priority, 100);

    Ok(())
}
