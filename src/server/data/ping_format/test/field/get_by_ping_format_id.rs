use super::*;

/// Tests getting fields for a ping format with multiple fields.
///
/// Verifies that the repository returns all fields for a ping format
/// ordered by priority in ascending order.
///
/// Expected: Ok with fields ordered by priority
#[tokio::test]
async fn gets_fields_ordered_by_priority() -> Result<(), DbErr> {
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

    // Create fields with different priorities (out of order)
    let field3 = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Field 3")
        .priority(3)
        .build()
        .await?;

    let field1 = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Field 1")
        .priority(1)
        .build()
        .await?;

    let field2 = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Field 2")
        .priority(2)
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.get_by_ping_format_id(ping_format.id).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 3);

    // Verify order by priority
    assert_eq!(fields[0].id, field1.id);
    assert_eq!(fields[0].priority, 1);
    assert_eq!(fields[1].id, field2.id);
    assert_eq!(fields[1].priority, 2);
    assert_eq!(fields[2].id, field3.id);
    assert_eq!(fields[2].priority, 3);

    Ok(())
}

/// Tests getting fields when ping format has no fields.
///
/// Verifies that the repository returns an empty vector when the
/// ping format has no associated fields.
///
/// Expected: Ok with empty vector
#[tokio::test]
async fn returns_empty_for_format_with_no_fields() -> Result<(), DbErr> {
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
    let result = repo.get_by_ping_format_id(ping_format.id).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 0);

    Ok(())
}

/// Tests getting fields for nonexistent ping format.
///
/// Verifies that the repository returns an empty vector when querying
/// fields for a ping_format_id that doesn't exist.
///
/// Expected: Ok with empty vector
#[tokio::test]
async fn returns_empty_for_nonexistent_format() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.get_by_ping_format_id(999999).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 0);

    Ok(())
}

/// Tests getting fields filters by ping_format_id correctly.
///
/// Verifies that only fields belonging to the specified ping format
/// are returned, not fields from other formats.
///
/// Expected: Ok with only fields from specified format
#[tokio::test]
async fn filters_by_ping_format_id() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::DiscordGuild)
        .with_table(entity::prelude::PingFormat)
        .with_table(entity::prelude::PingFormatField)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format1 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create fields for both formats
    let field1_format1 = factory::ping_format_field::create_ping_format_field(
        db,
        ping_format1.id,
        "Format 1 Field 1",
        1,
    )
    .await?;

    let field2_format1 = factory::ping_format_field::create_ping_format_field(
        db,
        ping_format1.id,
        "Format 1 Field 2",
        2,
    )
    .await?;

    factory::ping_format_field::create_ping_format_field(
        db,
        ping_format2.id,
        "Format 2 Field 1",
        1,
    )
    .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.get_by_ping_format_id(ping_format1.id).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 2);

    // Verify only format1 fields are returned
    assert_eq!(fields[0].id, field1_format1.id);
    assert_eq!(fields[1].id, field2_format1.id);
    assert!(fields.iter().all(|f| f.ping_format_id == ping_format1.id));

    Ok(())
}

/// Tests getting fields with same priority values.
///
/// Verifies that fields with identical priority values are handled
/// correctly (order may vary but all are returned).
///
/// Expected: Ok with all fields returned
#[tokio::test]
async fn handles_fields_with_same_priority() -> Result<(), DbErr> {
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

    // Create multiple fields with same priority
    let field1 = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Field A")
        .priority(1)
        .build()
        .await?;

    let field2 = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Field B")
        .priority(1)
        .build()
        .await?;

    let field3 = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Field C")
        .priority(2)
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.get_by_ping_format_id(ping_format.id).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 3);

    // Verify all fields are returned
    let ids: Vec<i32> = fields.iter().map(|f| f.id).collect();
    assert!(ids.contains(&field1.id));
    assert!(ids.contains(&field2.id));
    assert!(ids.contains(&field3.id));

    // Verify fields are grouped by priority (priority 1 fields come before priority 2)
    let priority_1_count = fields.iter().filter(|f| f.priority == 1).count();
    assert_eq!(priority_1_count, 2);
    assert_eq!(fields[2].id, field3.id);
    assert_eq!(fields[2].priority, 2);

    Ok(())
}

/// Tests getting fields with negative priorities.
///
/// Verifies that fields with negative priority values are correctly
/// ordered (negative values come before positive values).
///
/// Expected: Ok with fields ordered correctly including negative priorities
#[tokio::test]
async fn orders_negative_priorities_correctly() -> Result<(), DbErr> {
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

    // Create fields with negative, zero, and positive priorities
    let field_positive =
        factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
            .name("Positive")
            .priority(1)
            .build()
            .await?;

    let field_negative =
        factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
            .name("Negative")
            .priority(-1)
            .build()
            .await?;

    let field_zero = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Zero")
        .priority(0)
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.get_by_ping_format_id(ping_format.id).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 3);

    // Verify order: -1, 0, 1
    assert_eq!(fields[0].id, field_negative.id);
    assert_eq!(fields[0].priority, -1);
    assert_eq!(fields[1].id, field_zero.id);
    assert_eq!(fields[1].priority, 0);
    assert_eq!(fields[2].id, field_positive.id);
    assert_eq!(fields[2].priority, 1);

    Ok(())
}

/// Tests getting fields includes all field properties.
///
/// Verifies that the returned domain models include all field properties
/// including id, ping_format_id, name, priority, and default_value.
///
/// Expected: Ok with complete field data
#[tokio::test]
async fn returns_complete_field_data() -> Result<(), DbErr> {
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

    let created_field = factory::ping_format_field::PingFormatFieldFactory::new(db, ping_format.id)
        .name("Test Field")
        .priority(5)
        .default_value(Some("Default Value".to_string()))
        .build()
        .await?;

    let repo = PingFormatFieldRepository::new(db);
    let result = repo.get_by_ping_format_id(ping_format.id).await;

    assert!(result.is_ok());
    let fields = result.unwrap();
    assert_eq!(fields.len(), 1);

    let field = &fields[0];
    assert_eq!(field.id, created_field.id);
    assert_eq!(field.ping_format_id, created_field.ping_format_id);
    assert_eq!(field.name, "Test Field");
    assert_eq!(field.priority, 5);
    assert_eq!(field.default_value, Some("Default Value".to_string()));

    Ok(())
}
