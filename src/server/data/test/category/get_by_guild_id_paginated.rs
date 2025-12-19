use super::*;

/// Helper function to create a Discord guild role for testing
async fn create_guild_role(
    db: &DatabaseConnection,
    guild_id: &str,
    role_id: &str,
) -> Result<(), DbErr> {
    entity::discord_guild_role::ActiveModel {
        guild_id: ActiveValue::Set(guild_id.to_string()),
        role_id: ActiveValue::Set(role_id.to_string()),
        name: ActiveValue::Set(format!("Role {}", role_id)),
        color: ActiveValue::Set(String::new()),
        position: ActiveValue::Set(0),
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Helper function to create a Discord guild channel for testing
async fn create_guild_channel(
    db: &DatabaseConnection,
    guild_id: &str,
    channel_id: &str,
) -> Result<(), DbErr> {
    entity::discord_guild_channel::ActiveModel {
        guild_id: ActiveValue::Set(guild_id.to_string()),
        channel_id: ActiveValue::Set(channel_id.to_string()),
        name: ActiveValue::Set(format!("Channel {}", channel_id)),
        position: ActiveValue::Set(0),
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Tests getting paginated categories for a guild.
///
/// Verifies that the repository successfully retrieves categories with
/// their ping formats and counts of related entities, properly paginated.
///
/// Expected: Ok with categories and total count
#[tokio::test]
async fn gets_paginated_categories_for_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create three categories
    let repo = FleetCategoryRepository::new(db);
    for i in 1..=3 {
        repo.create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: format!("Category {}", i),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;
    }

    let result = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await;

    assert!(result.is_ok());
    let (categories, total) = result.unwrap();
    assert_eq!(categories.len(), 3);
    assert_eq!(total, 3);

    Ok(())
}

/// Tests pagination with multiple pages.
///
/// Verifies that the repository correctly paginates results when there
/// are more categories than the per_page limit.
///
/// Expected: Ok with correct page of categories
#[tokio::test]
async fn paginates_categories_correctly() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create 5 categories
    let repo = FleetCategoryRepository::new(db);
    for i in 1..=5 {
        repo.create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: format!("Category {}", i),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;
    }

    // Get first page (2 items)
    let (page1, total) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 0, 2)
        .await?;
    assert_eq!(page1.len(), 2);
    assert_eq!(total, 5);

    // Get second page (2 items)
    let (page2, total) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 1, 2)
        .await?;
    assert_eq!(page2.len(), 2);
    assert_eq!(total, 5);

    // Get third page (1 item)
    let (page3, total) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 2, 2)
        .await?;
    assert_eq!(page3.len(), 1);
    assert_eq!(total, 5);

    // Verify no overlap
    assert_ne!(page1[0].category.id, page2[0].category.id);
    assert_ne!(page2[0].category.id, page3[0].category.id);

    Ok(())
}

/// Tests categories are sorted by name.
///
/// Verifies that categories are returned in alphabetical order by name.
///
/// Expected: Ok with categories sorted alphabetically
#[tokio::test]
async fn sorts_categories_by_name() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);

    // Create categories in reverse alphabetical order
    repo.create(CreateFleetCategoryParams {
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Zebra".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    repo.create(CreateFleetCategoryParams {
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Alpha".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    repo.create(CreateFleetCategoryParams {
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Middle".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    let (categories, _) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await?;

    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0].category.name, "Alpha");
    assert_eq!(categories[1].category.name, "Middle");
    assert_eq!(categories[2].category.name, "Zebra");

    Ok(())
}

/// Tests getting categories with related entity counts.
///
/// Verifies that the repository correctly returns counts of access roles,
/// ping roles, and channels for each category.
///
/// Expected: Ok with accurate counts
#[tokio::test]
async fn returns_categories_with_counts() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create guild roles and channels first
    create_guild_role(db, &guild.guild_id, "1001").await?;
    create_guild_role(db, &guild.guild_id, "1002").await?;
    create_guild_role(db, &guild.guild_id, "2001").await?;
    create_guild_role(db, &guild.guild_id, "2002").await?;
    create_guild_role(db, &guild.guild_id, "2003").await?;
    create_guild_channel(db, &guild.guild_id, "3001").await?;
    create_guild_channel(db, &guild.guild_id, "3002").await?;
    create_guild_channel(db, &guild.guild_id, "3003").await?;
    create_guild_channel(db, &guild.guild_id, "3004").await?;

    let repo = FleetCategoryRepository::new(db);

    // Create category with various related entities
    repo.create(CreateFleetCategoryParams {
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Test Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![
            AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: false,
            },
            AccessRoleData {
                role_id: 1002,
                can_view: true,
                can_create: true,
                can_manage: false,
            },
        ],
        ping_roles: vec![2001, 2002, 2003],
        channels: vec![3001, 3002, 3003, 3004],
    })
    .await?;

    // Create category with no related entities
    repo.create(CreateFleetCategoryParams {
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Empty Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    let (categories, _) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await?;

    assert_eq!(categories.len(), 2);

    // Empty Category comes first alphabetically
    let empty_category = &categories[0];
    assert_eq!(empty_category.category.name, "Empty Category");
    assert_eq!(empty_category.access_roles_count, 0);
    assert_eq!(empty_category.ping_roles_count, 0);
    assert_eq!(empty_category.channels_count, 0);

    // Test Category comes second
    let test_category = &categories[1];
    assert_eq!(test_category.category.name, "Test Category");
    assert_eq!(test_category.access_roles_count, 2);
    assert_eq!(test_category.ping_roles_count, 3);
    assert_eq!(test_category.channels_count, 4);

    Ok(())
}

/// Tests filtering categories by guild ID.
///
/// Verifies that only categories belonging to the specified guild are
/// returned, not categories from other guilds.
///
/// Expected: Ok with only matching guild's categories
#[tokio::test]
async fn filters_categories_by_guild_id() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::discord_guild::create_guild(db).await?;
    let guild2 = factory::discord_guild::create_guild(db).await?;
    let ping_format1 = factory::ping_format::create_ping_format(db, &guild1.guild_id).await?;
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild2.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);

    // Create categories for guild1
    repo.create(CreateFleetCategoryParams {
        guild_id: guild1.guild_id.parse().unwrap(),
        ping_format_id: ping_format1.id,
        name: "Guild 1 Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    // Create categories for guild2
    repo.create(CreateFleetCategoryParams {
        guild_id: guild2.guild_id.parse().unwrap(),
        ping_format_id: ping_format2.id,
        name: "Guild 2 Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    // Query guild1 categories
    let (guild1_categories, guild1_total) = repo
        .get_by_guild_id_paginated(guild1.guild_id.parse().unwrap(), 0, 10)
        .await?;

    assert_eq!(guild1_categories.len(), 1);
    assert_eq!(guild1_total, 1);
    assert_eq!(guild1_categories[0].category.name, "Guild 1 Category");
    assert_eq!(guild1_categories[0].category.guild_id, guild1.guild_id);

    // Query guild2 categories
    let (guild2_categories, guild2_total) = repo
        .get_by_guild_id_paginated(guild2.guild_id.parse().unwrap(), 0, 10)
        .await?;

    assert_eq!(guild2_categories.len(), 1);
    assert_eq!(guild2_total, 1);
    assert_eq!(guild2_categories[0].category.name, "Guild 2 Category");
    assert_eq!(guild2_categories[0].category.guild_id, guild2.guild_id);

    Ok(())
}

/// Tests getting categories for guild with no categories.
///
/// Verifies that the repository returns an empty list and zero total
/// when querying a guild that has no categories.
///
/// Expected: Ok with empty list and total of 0
#[tokio::test]
async fn returns_empty_for_guild_without_categories() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;

    let repo = FleetCategoryRepository::new(db);
    let (categories, total) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await?;

    assert_eq!(categories.len(), 0);
    assert_eq!(total, 0);

    Ok(())
}

/// Tests categories include ping format data.
///
/// Verifies that each category in the paginated results includes its
/// associated ping format information.
///
/// Expected: Ok with categories containing ping format data
#[tokio::test]
async fn includes_ping_format_data() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);
    repo.create(CreateFleetCategoryParams {
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Test Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    let (categories, _) = repo
        .get_by_guild_id_paginated(guild.guild_id.parse().unwrap(), 0, 10)
        .await?;

    assert_eq!(categories.len(), 1);
    assert!(categories[0].ping_format.is_some());
    assert_eq!(
        categories[0].ping_format.as_ref().unwrap().id,
        ping_format.id
    );

    Ok(())
}
