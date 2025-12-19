use super::*;
use sea_orm::ActiveModelTrait;

/// Tests deleting an existing ping format field.
///
/// Verifies that the repository successfully deletes a field record
/// from the database.
///
/// Expected: Ok with field deleted
#[tokio::test]
async fn deletes_field() -> Result<(), DbErr> {
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

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.delete(field.id).await;

    assert!(result.is_ok());

    // Verify field no longer exists
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?;
    assert!(db_field.is_none());

    Ok(())
}

/// Tests deleting nonexistent field ID.
///
/// Verifies that the repository returns Ok even when attempting to delete
/// a field that doesn't exist (idempotent operation).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn succeeds_for_nonexistent_field() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.delete(999999).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting field doesn't affect other fields.
///
/// Verifies that deleting one field leaves other fields for the same
/// ping format intact.
///
/// Expected: Ok with only specified field deleted
#[tokio::test]
async fn deletes_only_specified_field() -> Result<(), DbErr> {
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
    let field3 =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Field 3", 3)
            .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.delete(field2.id).await;

    assert!(result.is_ok());

    // Verify field2 is deleted
    let db_field2 = entity::prelude::PingFormatField::find_by_id(field2.id)
        .one(db)
        .await?;
    assert!(db_field2.is_none());

    // Verify other fields still exist
    let db_field1 = entity::prelude::PingFormatField::find_by_id(field1.id)
        .one(db)
        .await?;
    assert!(db_field1.is_some());

    let db_field3 = entity::prelude::PingFormatField::find_by_id(field3.id)
        .one(db)
        .await?;
    assert!(db_field3.is_some());

    Ok(())
}

/// Tests deleting all fields for a ping format.
///
/// Verifies that multiple fields can be deleted and the format
/// can exist without any fields.
///
/// Expected: Ok with all fields deleted
#[tokio::test]
async fn deletes_all_fields_for_format() -> Result<(), DbErr> {
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
    repo.delete(field1.id).await?;
    repo.delete(field2.id).await?;

    // Verify no fields remain
    let count = entity::prelude::PingFormatField::find()
        .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format.id))
        .count(db)
        .await?;
    assert_eq!(count, 0);

    // Verify ping format still exists
    let db_format = entity::prelude::PingFormat::find_by_id(ping_format.id)
        .one(db)
        .await?;
    assert!(db_format.is_some());

    Ok(())
}

/// Tests cascade deletion of fleet field values.
///
/// Verifies that deleting a ping format field automatically deletes
/// associated fleet field values due to CASCADE foreign key constraint.
///
/// Expected: Ok with field and associated values deleted
#[tokio::test]
async fn cascades_to_fleet_field_values() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_message_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet with field values
    let (user, _guild, ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;
    let field =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Test Field", 1)
            .await?;

    let fleet = factory::fleet::FleetFactory::new(db, category.id, &user.discord_id)
        .build()
        .await?;

    // Create fleet field value
    entity::fleet_field_value::ActiveModel {
        fleet_id: sea_orm::ActiveValue::Set(fleet.id),
        field_id: sea_orm::ActiveValue::Set(field.id),
        value: sea_orm::ActiveValue::Set("Test Value".to_string()),
    }
    .insert(db)
    .await?;

    // Delete the field
    let repo = PingFormatFieldRepository::new(db);
    let result = repo.delete(field.id).await;

    assert!(result.is_ok());

    // Verify field is deleted
    let db_field = entity::prelude::PingFormatField::find_by_id(field.id)
        .one(db)
        .await?;
    assert!(db_field.is_none());

    // Verify fleet field value is also deleted (cascade)
    let db_value = entity::prelude::FleetFieldValue::find()
        .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
        .filter(entity::fleet_field_value::Column::FieldId.eq(field.id))
        .one(db)
        .await?;
    assert!(db_value.is_none());

    // Verify fleet still exists
    let db_fleet = entity::prelude::Fleet::find_by_id(fleet.id).one(db).await?;
    assert!(db_fleet.is_some());

    Ok(())
}

/// Tests deleting field multiple times is idempotent.
///
/// Verifies that calling delete on the same field ID multiple times
/// doesn't cause errors.
///
/// Expected: Ok on all delete calls
#[tokio::test]
async fn idempotent_delete() -> Result<(), DbErr> {
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

    let repo = PingFormatFieldRepository::new(db);

    // Delete first time
    let result1 = repo.delete(field.id).await;
    assert!(result1.is_ok());

    // Delete second time (already deleted)
    let result2 = repo.delete(field.id).await;
    assert!(result2.is_ok());

    // Delete third time
    let result3 = repo.delete(field.id).await;
    assert!(result3.is_ok());

    Ok(())
}

/// Tests deleting fields from different formats.
///
/// Verifies that deleting a field from one ping format doesn't affect
/// fields from other ping formats.
///
/// Expected: Ok with only specified format's field deleted
#[tokio::test]
async fn deletes_field_from_specific_format_only() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let format1 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let field_format1 =
        factory::ping_format_field::create_ping_format_field(db, format1.id, "Format 1 Field", 1)
            .await?;
    let field_format2 =
        factory::ping_format_field::create_ping_format_field(db, format2.id, "Format 2 Field", 1)
            .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.delete(field_format1.id).await;

    assert!(result.is_ok());

    // Verify format1's field is deleted
    let db_field1 = entity::prelude::PingFormatField::find_by_id(field_format1.id)
        .one(db)
        .await?;
    assert!(db_field1.is_none());

    // Verify format2's field still exists
    let db_field2 = entity::prelude::PingFormatField::find_by_id(field_format2.id)
        .one(db)
        .await?;
    assert!(db_field2.is_some());

    Ok(())
}
