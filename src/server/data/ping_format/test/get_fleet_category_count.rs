use super::*;

/// Tests getting count of fleet categories using a ping format.
///
/// Verifies that the repository correctly counts the number of fleet categories
/// that are configured to use a specific ping format.
///
/// Expected: Ok with correct count
#[tokio::test]
async fn gets_fleet_category_count() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(User)
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    // Create 3 categories using this ping format
    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
        .name("Category 1")
        .build()
        .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
        .name("Category 2")
        .build()
        .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
        .name("Category 3")
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo.get_fleet_category_count(ping_format.id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    Ok(())
}

/// Tests getting zero count for unused ping format.
///
/// Verifies that the repository returns zero when a ping format exists
/// but is not used by any fleet categories.
///
/// Expected: Ok(0)
#[tokio::test]
async fn returns_zero_for_unused_format() -> Result<(), DbErr> {
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

    let repo = PingFormatRepository::new(db);
    let result = repo.get_fleet_category_count(ping_format.id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    Ok(())
}

/// Tests getting count for nonexistent ping format.
///
/// Verifies that the repository returns zero when checking the usage count
/// of a ping format ID that doesn't exist in the database.
///
/// Expected: Ok(0)
#[tokio::test]
async fn returns_zero_for_nonexistent_format() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatRepository::new(db);
    let result = repo.get_fleet_category_count(999999).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    Ok(())
}

/// Tests counting categories with NULL ping_format_id.
///
/// Verifies that categories with NULL ping_format_id are not counted
/// when checking usage of a specific ping format.
///
/// Expected: Ok with only non-NULL categories counted
#[tokio::test]
async fn excludes_categories_with_null_format() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let format1 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild.guild_id).await?;

    // Create category using format1
    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format1.id)
        .name("Category with format 1")
        .build()
        .await?;

    // Create category using format2 (different format)
    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format2.id)
        .name("Category with format 2")
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo.get_fleet_category_count(format1.id).await;

    assert!(result.is_ok());
    // Should only count categories using format1
    assert_eq!(result.unwrap(), 1);

    Ok(())
}

/// Tests counting categories across multiple ping formats.
///
/// Verifies that the repository correctly counts only categories using the
/// specified format when multiple formats exist.
///
/// Expected: Ok with correct count per format
#[tokio::test]
async fn counts_categories_per_format() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let format1 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild.guild_id).await?;

    // Create 2 categories using format1
    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format1.id)
        .name("Category 1 with Format 1")
        .build()
        .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format1.id)
        .name("Category 2 with Format 1")
        .build()
        .await?;

    // Create 3 categories using format2
    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format2.id)
        .name("Category 1 with Format 2")
        .build()
        .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format2.id)
        .name("Category 2 with Format 2")
        .build()
        .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format2.id)
        .name("Category 3 with Format 2")
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let count1 = repo.get_fleet_category_count(format1.id).await?;
    let count2 = repo.get_fleet_category_count(format2.id).await?;

    assert_eq!(count1, 2);
    assert_eq!(count2, 3);

    Ok(())
}

/// Tests counting categories across multiple guilds.
///
/// Verifies that the repository counts all categories using a ping format,
/// even though ping formats should only be used within their own guild.
///
/// Expected: Ok with count including all categories using the format
#[tokio::test]
async fn counts_categories_across_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild1.guild_id).await?;

    // Create category in guild1 using the format
    factory::fleet_category::FleetCategoryFactory::new(db, &guild1.guild_id, ping_format.id)
        .build()
        .await?;

    // Create category in guild2 (theoretically shouldn't use guild1's format, but testing the count)
    factory::fleet_category::FleetCategoryFactory::new(db, &guild2.guild_id, ping_format.id)
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo.get_fleet_category_count(ping_format.id).await;

    assert!(result.is_ok());
    // Should count both categories, even from different guilds
    assert_eq!(result.unwrap(), 2);

    Ok(())
}

/// Tests count after deleting a category.
///
/// Verifies that the count decreases when a fleet category using the
/// ping format is deleted.
///
/// Expected: Ok with decreased count
#[tokio::test]
async fn decreases_count_after_category_deletion() -> Result<(), DbErr> {
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

    // Create 3 categories
    let category1 =
        factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
            .build()
            .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
        .build()
        .await?;

    factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);

    // Initial count
    let count_before = repo.get_fleet_category_count(ping_format.id).await?;
    assert_eq!(count_before, 3);

    // Delete one category
    entity::prelude::FleetCategory::delete_by_id(category1.id)
        .exec(db)
        .await?;

    // Count after deletion
    let count_after = repo.get_fleet_category_count(ping_format.id).await?;
    assert_eq!(count_after, 2);

    Ok(())
}

/// Tests count after changing category's ping format.
///
/// Verifies that the count updates correctly when a category changes from
/// one ping format to another.
///
/// Expected: Ok with updated counts for both formats
#[tokio::test]
async fn updates_count_after_category_format_change() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .with_table(FleetCategory)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let format1 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild.guild_id).await?;

    // Create category using format1
    let category =
        factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, format1.id)
            .build()
            .await?;

    let repo = PingFormatRepository::new(db);

    // Initial counts
    let count1_before = repo.get_fleet_category_count(format1.id).await?;
    let count2_before = repo.get_fleet_category_count(format2.id).await?;
    assert_eq!(count1_before, 1);
    assert_eq!(count2_before, 0);

    // Change category to use format2
    use sea_orm::ActiveModelTrait;
    let mut active_category: entity::fleet_category::ActiveModel = category.into();
    active_category.ping_format_id = sea_orm::ActiveValue::Set(format2.id);
    active_category.update(db).await?;

    // Counts after change
    let count1_after = repo.get_fleet_category_count(format1.id).await?;
    let count2_after = repo.get_fleet_category_count(format2.id).await?;
    assert_eq!(count1_after, 0);
    assert_eq!(count2_after, 1);

    Ok(())
}

/// Tests count with large number of categories.
///
/// Verifies that the repository correctly counts when a ping format is used
/// by many fleet categories.
///
/// Expected: Ok with correct count
#[tokio::test]
async fn counts_large_number_of_categories() -> Result<(), DbErr> {
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

    // Create 20 categories using this ping format
    for i in 1..=20 {
        factory::fleet_category::FleetCategoryFactory::new(db, &guild.guild_id, ping_format.id)
            .name(format!("Category {}", i))
            .build()
            .await?;
    }

    let repo = PingFormatRepository::new(db);
    let result = repo.get_fleet_category_count(ping_format.id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 20);

    Ok(())
}
