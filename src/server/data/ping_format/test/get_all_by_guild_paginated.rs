use super::*;

/// Tests getting first page of ping formats.
///
/// Verifies that the repository returns the correct ping formats for the first page
/// when multiple formats exist for a guild, ordered alphabetically by name.
///
/// Expected: Ok with first page of formats and correct total count
#[tokio::test]
async fn gets_first_page_of_formats() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create 5 formats with names that will be ordered alphabetically
    for i in 1..=5 {
        factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
            .name(format!("Format {}", i))
            .build()
            .await?;
    }

    let repo = PingFormatRepository::new(db);
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 0, 3)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 3);
    assert_eq!(total, 5);
    assert_eq!(formats[0].name, "Format 1");
    assert_eq!(formats[1].name, "Format 2");
    assert_eq!(formats[2].name, "Format 3");

    Ok(())
}

/// Tests getting second page of ping formats.
///
/// Verifies that pagination correctly returns the second page of results
/// with the remaining formats.
///
/// Expected: Ok with second page of formats
#[tokio::test]
async fn gets_second_page_of_formats() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create 5 formats
    for i in 1..=5 {
        factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
            .name(format!("Format {}", i))
            .build()
            .await?;
    }

    let repo = PingFormatRepository::new(db);
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 1, 3)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 2);
    assert_eq!(total, 5);
    assert_eq!(formats[0].name, "Format 4");
    assert_eq!(formats[1].name, "Format 5");

    Ok(())
}

/// Tests getting formats ordered alphabetically by name.
///
/// Verifies that ping formats are returned in alphabetical order by name,
/// regardless of creation order.
///
/// Expected: Ok with formats ordered alphabetically
#[tokio::test]
async fn orders_formats_alphabetically() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create formats in non-alphabetical order
    factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
        .name("Zebra")
        .build()
        .await?;
    factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
        .name("Apple")
        .build()
        .await?;
    factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
        .name("Mango")
        .build()
        .await?;
    factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
        .name("Banana")
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(total, 4);
    assert_eq!(formats.len(), 4);
    assert_eq!(formats[0].name, "Apple");
    assert_eq!(formats[1].name, "Banana");
    assert_eq!(formats[2].name, "Mango");
    assert_eq!(formats[3].name, "Zebra");

    Ok(())
}

/// Tests filtering formats by guild ID.
///
/// Verifies that only ping formats belonging to the specified guild are returned,
/// and formats from other guilds are excluded.
///
/// Expected: Ok with only formats for specified guild
#[tokio::test]
async fn filters_formats_by_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;

    // Create formats for guild1
    factory::ping_format::PingFormatFactory::new(db, &guild1.guild_id)
        .name("Guild 1 Format A")
        .build()
        .await?;
    factory::ping_format::PingFormatFactory::new(db, &guild1.guild_id)
        .name("Guild 1 Format B")
        .build()
        .await?;

    // Create formats for guild2
    factory::ping_format::PingFormatFactory::new(db, &guild2.guild_id)
        .name("Guild 2 Format A")
        .build()
        .await?;
    factory::ping_format::PingFormatFactory::new(db, &guild2.guild_id)
        .name("Guild 2 Format B")
        .build()
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .get_all_by_guild_paginated(guild1.guild_id.parse().unwrap(), 0, 10)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(total, 2);
    assert_eq!(formats.len(), 2);
    assert!(formats.iter().all(|f| f.guild_id == guild1.guild_id));
    assert_eq!(formats[0].name, "Guild 1 Format A");
    assert_eq!(formats[1].name, "Guild 1 Format B");

    Ok(())
}

/// Tests getting formats for guild with no formats.
///
/// Verifies that querying a guild that has no ping formats returns an empty
/// vector with a total count of zero.
///
/// Expected: Ok with empty vector and zero count
#[tokio::test]
async fn returns_empty_for_guild_without_formats() -> Result<(), DbErr> {
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
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 0);
    assert_eq!(total, 0);

    Ok(())
}

/// Tests getting formats for nonexistent guild.
///
/// Verifies that querying a guild ID that doesn't exist returns an empty result
/// rather than an error.
///
/// Expected: Ok with empty vector and zero count
#[tokio::test]
async fn returns_empty_for_nonexistent_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = PingFormatRepository::new(db);
    let result = repo.get_all_by_guild_paginated(999999999, 0, 10).await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 0);
    assert_eq!(total, 0);

    Ok(())
}

/// Tests requesting page beyond available pages.
///
/// Verifies that requesting a page number beyond the available data returns
/// an empty vector while still reporting the correct total count.
///
/// Expected: Ok with empty vector but correct total count
#[tokio::test]
async fn returns_empty_for_page_beyond_available() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create 3 formats
    for i in 1..=3 {
        factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
            .name(format!("Format {}", i))
            .build()
            .await?;
    }

    let repo = PingFormatRepository::new(db);
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 5, 10)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 0);
    assert_eq!(total, 3);

    Ok(())
}

/// Tests pagination with single item per page.
///
/// Verifies that pagination works correctly when per_page is set to 1,
/// returning only a single format per page.
///
/// Expected: Ok with single format per page
#[tokio::test]
async fn handles_single_item_per_page() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create 3 formats
    for i in 1..=3 {
        factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
            .name(format!("Format {}", i))
            .build()
            .await?;
    }

    let repo = PingFormatRepository::new(db);

    // Get page 0
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 0, 1)
        .await;
    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 1);
    assert_eq!(total, 3);
    assert_eq!(formats[0].name, "Format 1");

    // Get page 1
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 1, 1)
        .await;
    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 1);
    assert_eq!(total, 3);
    assert_eq!(formats[0].name, "Format 2");

    // Get page 2
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 2, 1)
        .await;
    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 1);
    assert_eq!(total, 3);
    assert_eq!(formats[0].name, "Format 3");

    Ok(())
}

/// Tests pagination with per_page larger than total items.
///
/// Verifies that when per_page exceeds the total number of formats, all formats
/// are returned on the first page.
///
/// Expected: Ok with all formats on first page
#[tokio::test]
async fn handles_per_page_larger_than_total() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    // Create 3 formats
    for i in 1..=3 {
        factory::ping_format::PingFormatFactory::new(db, &guild.guild_id)
            .name(format!("Format {}", i))
            .build()
            .await?;
    }

    let repo = PingFormatRepository::new(db);
    let result = repo
        .get_all_by_guild_paginated(guild.guild_id.parse().unwrap(), 0, 100)
        .await;

    assert!(result.is_ok());
    let (formats, total) = result.unwrap();
    assert_eq!(formats.len(), 3);
    assert_eq!(total, 3);

    Ok(())
}
