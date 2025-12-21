use super::*;

/// Tests creating a new ping format.
///
/// Verifies that the repository successfully creates a new ping format record
/// with the specified guild ID and name.
///
/// Expected: Ok with ping format created
#[tokio::test]
async fn creates_ping_format() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .create(CreatePingFormatParam {
            guild_id: guild.guild_id.parse().unwrap(),
            name: "Test Format".to_string(),
        })
        .await;

    assert!(result.is_ok());
    let ping_format = result.unwrap();
    assert_eq!(ping_format.name, "Test Format");
    assert_eq!(ping_format.guild_id.to_string(), guild.guild_id);
    assert!(ping_format.id > 0);

    // Verify ping format exists in database
    let db_format = entity::prelude::PingFormat::find_by_id(ping_format.id)
        .one(db)
        .await?;
    assert!(db_format.is_some());
    assert_eq!(db_format.unwrap().name, "Test Format");

    Ok(())
}

/// Tests creating multiple ping formats for the same guild.
///
/// Verifies that the repository successfully creates multiple distinct ping format
/// records for a single guild, each with unique IDs and names.
///
/// Expected: Ok with multiple formats created
#[tokio::test]
async fn creates_multiple_formats_for_same_guild() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let repo = PingFormatRepository::new(db);

    let format1 = repo
        .create(CreatePingFormatParam {
            guild_id: guild.guild_id.parse().unwrap(),
            name: "Format 1".to_string(),
        })
        .await?;

    let format2 = repo
        .create(CreatePingFormatParam {
            guild_id: guild.guild_id.parse().unwrap(),
            name: "Format 2".to_string(),
        })
        .await?;

    let format3 = repo
        .create(CreatePingFormatParam {
            guild_id: guild.guild_id.parse().unwrap(),
            name: "Format 3".to_string(),
        })
        .await?;

    assert_ne!(format1.id, format2.id);
    assert_ne!(format1.id, format3.id);
    assert_ne!(format2.id, format3.id);
    assert_eq!(format1.guild_id, format2.guild_id);
    assert_eq!(format1.guild_id, format3.guild_id);

    // Verify all formats exist in database
    let count = entity::prelude::PingFormat::find().count(db).await?;
    assert_eq!(count, 3);

    Ok(())
}

/// Tests creating ping formats for different guilds.
///
/// Verifies that the repository correctly associates ping formats with their
/// respective guilds and that formats from different guilds are independent.
///
/// Expected: Ok with formats created for different guilds
#[tokio::test]
async fn creates_formats_for_different_guilds() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;
    let repo = PingFormatRepository::new(db);

    let format1 = repo
        .create(CreatePingFormatParam {
            guild_id: guild1.guild_id.parse().unwrap(),
            name: "Guild 1 Format".to_string(),
        })
        .await?;

    let format2 = repo
        .create(CreatePingFormatParam {
            guild_id: guild2.guild_id.parse().unwrap(),
            name: "Guild 2 Format".to_string(),
        })
        .await?;

    assert_ne!(format1.id, format2.id);
    assert_eq!(format1.guild_id.to_string(), guild1.guild_id);
    assert_eq!(format2.guild_id.to_string(), guild2.guild_id);
    assert_ne!(format1.guild_id, format2.guild_id);

    Ok(())
}

/// Tests creating a ping format with empty name.
///
/// Verifies that the repository allows creating ping formats with empty names,
/// as name validation is handled at the service/controller layer.
///
/// Expected: Ok with empty name accepted
#[tokio::test]
async fn creates_format_with_empty_name() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let repo = PingFormatRepository::new(db);

    let result = repo
        .create(CreatePingFormatParam {
            guild_id: guild.guild_id.parse().unwrap(),
            name: "".to_string(),
        })
        .await;

    assert!(result.is_ok());
    let format = result.unwrap();
    assert_eq!(format.name, "");

    Ok(())
}

/// Tests creating a ping format with long name.
///
/// Verifies that the repository can handle ping formats with long names,
/// ensuring no truncation or errors occur at the data layer.
///
/// Expected: Ok with full name preserved
#[tokio::test]
async fn creates_format_with_long_name() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let repo = PingFormatRepository::new(db);
    let long_name = "A".repeat(255);

    let result = repo
        .create(CreatePingFormatParam {
            guild_id: guild.guild_id.parse().unwrap(),
            name: long_name.clone(),
        })
        .await;

    assert!(result.is_ok());
    let format = result.unwrap();
    assert_eq!(format.name, long_name);
    assert_eq!(format.name.len(), 255);

    Ok(())
}

/// Tests creating a ping format for nonexistent guild.
///
/// Verifies that the repository returns a foreign key constraint error when
/// attempting to create a ping format for a guild that doesn't exist in the database.
///
/// Expected: Err with foreign key constraint error
#[tokio::test]
async fn fails_for_nonexistent_guild() -> Result<(), AppError> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatRepository::new(db);
    let result = repo
        .create(CreatePingFormatParam {
            guild_id: 999999999,
            name: "Test Format".to_string(),
        })
        .await;

    assert!(result.is_err());

    Ok(())
}
