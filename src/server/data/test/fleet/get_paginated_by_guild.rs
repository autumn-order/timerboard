use super::*;

/// Tests retrieving paginated fleets for a guild.
///
/// Verifies that the repository successfully retrieves fleets for a guild
/// with pagination and returns the correct total count.
///
/// Expected: Ok((fleets, total))
#[tokio::test]
async fn returns_paginated_fleets() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, guild, _ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    // Create multiple fleets
    let fleet_time1 = Utc::now() + Duration::hours(1);
    let fleet_time2 = Utc::now() + Duration::hours(2);
    let fleet_time3 = Utc::now() + Duration::hours(3);

    let repo = FleetRepository::new(db);
    repo.create(CreateFleetParams {
        category_id: category.id,
        name: "Fleet 1".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: fleet_time1,
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    repo.create(CreateFleetParams {
        category_id: category.id,
        name: "Fleet 2".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: fleet_time2,
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    repo.create(CreateFleetParams {
        category_id: category.id,
        name: "Fleet 3".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: fleet_time3,
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    let result = repo
        .get_paginated_by_guild(guild.guild_id.parse().unwrap(), 0, 10, None)
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert_eq!(fleets.len(), 3);
    assert_eq!(total, 3);

    // Verify ordering (ascending by fleet_time)
    assert_eq!(fleets[0].name, "Fleet 1");
    assert_eq!(fleets[1].name, "Fleet 2");
    assert_eq!(fleets[2].name, "Fleet 3");

    Ok(())
}

/// Tests pagination works correctly.
///
/// Verifies that pagination returns the correct subset of results
/// based on page number and page size.
///
/// Expected: Ok with correct page of results
#[tokio::test]
async fn respects_pagination_parameters() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, guild, _ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    let repo = FleetRepository::new(db);

    // Create 5 fleets
    for i in 1..=5 {
        repo.create(CreateFleetParams {
            category_id: category.id,
            name: format!("Fleet {}", i),
            commander_id: user.discord_id.parse().unwrap(),
            fleet_time: Utc::now() + Duration::hours(i),
            description: None,
            hidden: false,
            disable_reminder: false,
            field_values: HashMap::new(),
        })
        .await?;
    }

    // Get first page (2 per page)
    let result = repo
        .get_paginated_by_guild(guild.guild_id.parse().unwrap(), 0, 2, None)
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert_eq!(fleets.len(), 2);
    assert_eq!(total, 5);
    assert_eq!(fleets[0].name, "Fleet 1");
    assert_eq!(fleets[1].name, "Fleet 2");

    // Get second page
    let result = repo
        .get_paginated_by_guild(guild.guild_id.parse().unwrap(), 1, 2, None)
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert_eq!(fleets.len(), 2);
    assert_eq!(total, 5);
    assert_eq!(fleets[0].name, "Fleet 3");
    assert_eq!(fleets[1].name, "Fleet 4");

    Ok(())
}

/// Tests filtering by viewable category IDs.
///
/// Verifies that only fleets in categories the user can view are returned
/// when viewable_category_ids is provided.
///
/// Expected: Ok with only fleets from viewable categories
#[tokio::test]
async fn filters_by_viewable_categories() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, guild, ping_format) = factory::helpers::create_guild_dependencies(db).await?;

    // Create two categories
    let category1 =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;
    let category2 =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    let repo = FleetRepository::new(db);

    // Create fleet in category1
    repo.create(CreateFleetParams {
        category_id: category1.id,
        name: "Fleet 1".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: Utc::now() + Duration::hours(1),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    // Create fleet in category2
    repo.create(CreateFleetParams {
        category_id: category2.id,
        name: "Fleet 2".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: Utc::now() + Duration::hours(2),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    // Query with only category1 viewable
    let result = repo
        .get_paginated_by_guild(
            guild.guild_id.parse().unwrap(),
            0,
            10,
            Some(vec![category1.id]),
        )
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert_eq!(fleets.len(), 1);
    assert_eq!(total, 1);
    assert_eq!(fleets[0].name, "Fleet 1");
    assert_eq!(fleets[0].category_id, category1.id);

    Ok(())
}

/// Tests empty viewable categories returns no fleets.
///
/// Verifies that when an empty category list is provided, no fleets
/// are returned even if fleets exist.
///
/// Expected: Ok with empty results
#[tokio::test]
async fn returns_empty_for_empty_viewable_categories() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet with all dependencies
    let (_user, guild, _ping_format, _category, _fleet) =
        factory::helpers::create_fleet_with_dependencies(db).await?;

    let repo = FleetRepository::new(db);
    let result = repo
        .get_paginated_by_guild(guild.guild_id.parse().unwrap(), 0, 10, Some(vec![]))
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert!(fleets.is_empty());
    assert_eq!(total, 0);

    Ok(())
}

/// Tests cutoff time filters old fleets.
///
/// Verifies that fleets older than 1 hour are not returned in the results.
///
/// Expected: Ok with only recent/upcoming fleets
#[tokio::test]
async fn excludes_fleets_older_than_one_hour() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, guild, _ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    let repo = FleetRepository::new(db);

    // Create old fleet (2 hours ago - should be excluded)
    repo.create(CreateFleetParams {
        category_id: category.id,
        name: "Old Fleet".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: Utc::now() - Duration::hours(2),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    // Create recent fleet (30 minutes ago - should be included)
    repo.create(CreateFleetParams {
        category_id: category.id,
        name: "Recent Fleet".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: Utc::now() - Duration::minutes(30),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    // Create upcoming fleet (should be included)
    repo.create(CreateFleetParams {
        category_id: category.id,
        name: "Upcoming Fleet".to_string(),
        commander_id: user.discord_id.parse().unwrap(),
        fleet_time: Utc::now() + Duration::hours(1),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    let result = repo
        .get_paginated_by_guild(guild.guild_id.parse().unwrap(), 0, 10, None)
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert_eq!(fleets.len(), 2);
    assert_eq!(total, 2);

    let names: Vec<&str> = fleets.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"Recent Fleet"));
    assert!(names.contains(&"Upcoming Fleet"));
    assert!(!names.contains(&"Old Fleet"));

    Ok(())
}

/// Tests returns empty for guild with no fleets.
///
/// Verifies that querying a guild with no fleets returns an empty vector.
///
/// Expected: Ok with empty results
#[tokio::test]
async fn returns_empty_for_guild_with_no_fleets() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create guild but no fleets
    let (_user, guild, _ping_format) = factory::helpers::create_guild_dependencies(db).await?;

    let repo = FleetRepository::new(db);
    let result = repo
        .get_paginated_by_guild(guild.guild_id.parse().unwrap(), 0, 10, None)
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert!(fleets.is_empty());
    assert_eq!(total, 0);

    Ok(())
}

/// Tests fleets are isolated per guild.
///
/// Verifies that querying one guild does not return fleets from other guilds.
///
/// Expected: Ok with only fleets from specified guild
#[tokio::test]
async fn returns_only_fleets_for_specified_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create first guild with fleet
    let (user1, guild1, _ping_format1, category1) =
        factory::helpers::create_fleet_dependencies(db).await?;

    // Create second guild with fleet
    let user2 = factory::user::UserFactory::new(db)
        .discord_id("987654321")
        .build()
        .await?;
    let guild2 = factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("111222333")
        .build()
        .await?;
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild2.guild_id).await?;
    let category2 =
        factory::fleet_category::create_category(db, &guild2.guild_id, ping_format2.id).await?;

    let repo = FleetRepository::new(db);

    // Create fleet in guild1
    repo.create(CreateFleetParams {
        category_id: category1.id,
        name: "Guild 1 Fleet".to_string(),
        commander_id: user1.discord_id.parse().unwrap(),
        fleet_time: Utc::now() + Duration::hours(1),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    // Create fleet in guild2
    repo.create(CreateFleetParams {
        category_id: category2.id,
        name: "Guild 2 Fleet".to_string(),
        commander_id: user2.discord_id.parse().unwrap(),
        fleet_time: Utc::now() + Duration::hours(2),
        description: None,
        hidden: false,
        disable_reminder: false,
        field_values: HashMap::new(),
    })
    .await?;

    // Query guild1
    let result = repo
        .get_paginated_by_guild(guild1.guild_id.parse().unwrap(), 0, 10, None)
        .await;

    assert!(result.is_ok());
    let (fleets, total) = result.unwrap();
    assert_eq!(fleets.len(), 1);
    assert_eq!(total, 1);
    assert_eq!(fleets[0].name, "Guild 1 Fleet");

    Ok(())
}
