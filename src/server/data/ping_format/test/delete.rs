use super::*;

/// Tests deleting a ping format successfully.
///
/// Verifies that the repository successfully deletes an existing ping format
/// and that it no longer exists in the database.
///
/// Expected: Ok with format deleted from database
#[tokio::test]
async fn deletes_format_successfully() -> Result<(), DbErr> {
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

    let repo = PingFormatRepository::new(db);
    let result = repo.delete(format_id).await;

    assert!(result.is_ok());

    // Verify format no longer exists in database
    let db_format = entity::prelude::PingFormat::find_by_id(format_id)
        .one(db)
        .await?;
    assert!(db_format.is_none());

    Ok(())
}

/// Tests deleting a nonexistent ping format.
///
/// Verifies that attempting to delete a ping format that doesn't exist
/// succeeds without error (idempotent operation).
///
/// Expected: Ok without error
#[tokio::test]
async fn deletes_nonexistent_format_succeeds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatRepository::new(db);
    let result = repo.delete(999999).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting a ping format multiple times.
///
/// Verifies that deleting the same ping format ID multiple times succeeds
/// without error (idempotent operation).
///
/// Expected: Ok on all delete attempts
#[tokio::test]
async fn deletes_format_multiple_times_succeeds() -> Result<(), DbErr> {
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

    let repo = PingFormatRepository::new(db);

    // Delete first time
    let result1 = repo.delete(format_id).await;
    assert!(result1.is_ok());

    // Delete second time
    let result2 = repo.delete(format_id).await;
    assert!(result2.is_ok());

    // Delete third time
    let result3 = repo.delete(format_id).await;
    assert!(result3.is_ok());

    Ok(())
}

/// Tests deleting one format doesn't affect others.
///
/// Verifies that deleting a specific ping format doesn't delete or affect
/// other ping formats in the database.
///
/// Expected: Ok with only target format deleted
#[tokio::test]
async fn deletes_format_without_affecting_others() -> Result<(), DbErr> {
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
    let result = repo.delete(format2.id).await;

    assert!(result.is_ok());

    // Verify format2 deleted
    let db_format2 = entity::prelude::PingFormat::find_by_id(format2.id)
        .one(db)
        .await?;
    assert!(db_format2.is_none());

    // Verify format1 and format3 still exist
    let db_format1 = entity::prelude::PingFormat::find_by_id(format1.id)
        .one(db)
        .await?;
    assert!(db_format1.is_some());

    let db_format3 = entity::prelude::PingFormat::find_by_id(format3.id)
        .one(db)
        .await?;
    assert!(db_format3.is_some());

    Ok(())
}

/// Tests deleting ping format cascades to ping format fields.
///
/// Verifies that deleting a ping format automatically deletes all associated
/// ping format fields due to CASCADE foreign key constraint.
///
/// Expected: Ok with fields also deleted
#[tokio::test]
async fn deletes_format_cascades_to_fields() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    // Create ping format fields
    use crate::server::data::ping_format::field::PingFormatFieldRepository;
    use crate::server::model::ping_format::CreatePingFormatFieldParam;

    let field_repo = PingFormatFieldRepository::new(db);
    let field1 = field_repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Field 1".to_string(),
            priority: 0,
            default_value: None,
        })
        .await?;

    let field2 = field_repo
        .create(CreatePingFormatFieldParam {
            ping_format_id: ping_format.id,
            name: "Field 2".to_string(),
            priority: 1,
            default_value: None,
        })
        .await?;

    // Delete the ping format
    let repo = PingFormatRepository::new(db);
    let result = repo.delete(ping_format.id).await;

    assert!(result.is_ok());

    // Verify ping format deleted
    let db_format = entity::prelude::PingFormat::find_by_id(ping_format.id)
        .one(db)
        .await?;
    assert!(db_format.is_none());

    // Verify fields also deleted (CASCADE)
    let db_field1 = entity::prelude::PingFormatField::find_by_id(field1.id)
        .one(db)
        .await?;
    assert!(db_field1.is_none());

    let db_field2 = entity::prelude::PingFormatField::find_by_id(field2.id)
        .one(db)
        .await?;
    assert!(db_field2.is_none());

    Ok(())
}

/// Tests deleting ping format cascades to fleet categories.
///
/// Verifies that deleting a ping format that is used by fleet categories
/// also deletes those categories due to CASCADE foreign key constraint.
///
/// Expected: Ok with category also deleted
#[tokio::test]
async fn deletes_format_cascades_to_categories() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    // Create a fleet category using this ping format
    let category =
        factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
            .build()
            .await?;

    // Delete the ping format
    let repo = PingFormatRepository::new(db);
    let result = repo.delete(ping_format.id).await;

    assert!(result.is_ok());

    // Verify category also deleted (CASCADE)
    let db_category = entity::prelude::FleetCategory::find_by_id(category.id)
        .one(db)
        .await?;
    assert!(db_category.is_none());

    Ok(())
}

/// Tests deleting ping format from different guilds.
///
/// Verifies that ping formats from different guilds can be deleted independently
/// and that deleting one guild's format doesn't affect another guild's formats.
///
/// Expected: Ok with only target guild's format deleted
#[tokio::test]
async fn deletes_format_from_different_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;

    let format1 = factory::create_ping_format(db, &guild1.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild2.guild_id).await?;

    // Delete guild1's format
    let repo = PingFormatRepository::new(db);
    let result = repo.delete(format1.id).await;

    assert!(result.is_ok());

    // Verify format1 deleted
    let db_format1 = entity::prelude::PingFormat::find_by_id(format1.id)
        .one(db)
        .await?;
    assert!(db_format1.is_none());

    // Verify format2 still exists
    let db_format2 = entity::prelude::PingFormat::find_by_id(format2.id)
        .one(db)
        .await?;
    assert!(db_format2.is_some());

    Ok(())
}

/// Tests deleting all formats for a guild.
///
/// Verifies that all ping formats for a guild can be deleted sequentially
/// without errors.
///
/// Expected: Ok with all formats deleted
#[tokio::test]
async fn deletes_all_formats_for_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create multiple formats
    let format1 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format3 = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);

    // Delete all formats
    repo.delete(format1.id).await?;
    repo.delete(format2.id).await?;
    repo.delete(format3.id).await?;

    // Verify all formats deleted
    let count = entity::prelude::PingFormat::find().count(db).await?;
    assert_eq!(count, 0);

    Ok(())
}
