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

/// Tests creating a new category without any related entities.
///
/// Verifies that the repository successfully creates a new fleet category record
/// with the specified guild_id, ping_format_id, name, and duration fields but
/// no access roles, ping roles, or channels.
///
/// Expected: Ok with category created
#[tokio::test]
async fn creates_category_without_related_entities() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);
    let result = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: Some(Duration::minutes(30)),
            ping_reminder: Some(Duration::minutes(15)),
            max_pre_ping: Some(Duration::hours(2)),
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await;

    assert!(result.is_ok());
    let category = result.unwrap();
    assert_eq!(category.name, "Test Category");
    assert_eq!(category.ping_format_id, ping_format.id);
    assert_eq!(category.ping_lead_time, Some(Duration::minutes(30)));
    assert_eq!(category.ping_reminder, Some(Duration::minutes(15)));
    assert_eq!(category.max_pre_ping, Some(Duration::hours(2)));

    // Verify category exists in database
    let db_category = entity::prelude::FleetCategory::find_by_id(category.id)
        .one(db)
        .await?;
    assert!(db_category.is_some());
    assert_eq!(db_category.unwrap().name, "Test Category");

    Ok(())
}

/// Tests creating a category with access roles.
///
/// Verifies that the repository successfully creates a category and its
/// associated access roles with proper permission flags.
///
/// Expected: Ok with category and access roles created
#[tokio::test]
async fn creates_category_with_access_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create guild roles first
    create_guild_role(db, &guild.guild_id, "1001").await?;
    create_guild_role(db, &guild.guild_id, "1002").await?;

    let access_role1 = AccessRoleData {
        role_id: 1001,
        can_view: true,
        can_create: true,
        can_manage: false,
    };
    let access_role2 = AccessRoleData {
        role_id: 1002,
        can_view: true,
        can_create: false,
        can_manage: true,
    };

    let repo = FleetCategoryRepository::new(db);
    let result = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![access_role1, access_role2],
            ping_roles: vec![],
            channels: vec![],
        })
        .await;

    assert!(result.is_ok());
    let category = result.unwrap();

    // Verify access roles were created
    let db_access_roles = entity::prelude::FleetCategoryAccessRole::find()
        .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category.id))
        .all(db)
        .await?;

    assert_eq!(db_access_roles.len(), 2);

    let role1 = db_access_roles
        .iter()
        .find(|r| r.role_id == "1001")
        .unwrap();
    assert!(role1.can_view);
    assert!(role1.can_create);
    assert!(!role1.can_manage);

    let role2 = db_access_roles
        .iter()
        .find(|r| r.role_id == "1002")
        .unwrap();
    assert!(role2.can_view);
    assert!(!role2.can_create);
    assert!(role2.can_manage);

    Ok(())
}

/// Tests creating a category with ping roles.
///
/// Verifies that the repository successfully creates a category and its
/// associated ping roles.
///
/// Expected: Ok with category and ping roles created
#[tokio::test]
async fn creates_category_with_ping_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create guild roles first
    create_guild_role(db, &guild.guild_id, "2001").await?;
    create_guild_role(db, &guild.guild_id, "2002").await?;
    create_guild_role(db, &guild.guild_id, "2003").await?;

    let repo = FleetCategoryRepository::new(db);
    let result = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![2001, 2002, 2003],
            channels: vec![],
        })
        .await;

    assert!(result.is_ok());
    let category = result.unwrap();

    // Verify ping roles were created
    let db_ping_roles = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category.id))
        .all(db)
        .await?;

    assert_eq!(db_ping_roles.len(), 3);
    let role_ids: Vec<String> = db_ping_roles.iter().map(|r| r.role_id.clone()).collect();
    assert!(role_ids.contains(&"2001".to_string()));
    assert!(role_ids.contains(&"2002".to_string()));
    assert!(role_ids.contains(&"2003".to_string()));

    Ok(())
}

/// Tests creating a category with channels.
///
/// Verifies that the repository successfully creates a category and its
/// associated channels.
///
/// Expected: Ok with category and channels created
#[tokio::test]
async fn creates_category_with_channels() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create guild channels first
    create_guild_channel(db, &guild.guild_id, "3001").await?;
    create_guild_channel(db, &guild.guild_id, "3002").await?;

    let repo = FleetCategoryRepository::new(db);
    let result = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![3001, 3002],
        })
        .await;

    assert!(result.is_ok());
    let category = result.unwrap();

    // Verify channels were created
    let db_channels = entity::prelude::FleetCategoryChannel::find()
        .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(category.id))
        .all(db)
        .await?;

    assert_eq!(db_channels.len(), 2);
    let channel_ids: Vec<String> = db_channels.iter().map(|c| c.channel_id.clone()).collect();
    assert!(channel_ids.contains(&"3001".to_string()));
    assert!(channel_ids.contains(&"3002".to_string()));

    Ok(())
}

/// Tests creating a category with all related entities.
///
/// Verifies that the repository successfully creates a category along with
/// access roles, ping roles, and channels in a single operation.
///
/// Expected: Ok with category and all related entities created
#[tokio::test]
async fn creates_category_with_all_related_entities() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create guild role and channels first
    create_guild_role(db, &guild.guild_id, "1001").await?;
    create_guild_role(db, &guild.guild_id, "2001").await?;
    create_guild_role(db, &guild.guild_id, "2002").await?;
    create_guild_channel(db, &guild.guild_id, "3001").await?;
    create_guild_channel(db, &guild.guild_id, "3002").await?;
    create_guild_channel(db, &guild.guild_id, "3003").await?;

    let access_role = AccessRoleData {
        role_id: 1001,
        can_view: true,
        can_create: true,
        can_manage: true,
    };

    let repo = FleetCategoryRepository::new(db);
    let result = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Full Category".to_string(),
            ping_lead_time: Some(Duration::minutes(45)),
            ping_reminder: Some(Duration::minutes(10)),
            max_pre_ping: Some(Duration::hours(3)),
            access_roles: vec![access_role],
            ping_roles: vec![2001, 2002],
            channels: vec![3001, 3002, 3003],
        })
        .await;

    assert!(result.is_ok());
    let category = result.unwrap();
    assert_eq!(category.name, "Full Category");

    // Verify all related entities
    let access_roles_count = entity::prelude::FleetCategoryAccessRole::find()
        .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(category.id))
        .count(db)
        .await?;
    assert_eq!(access_roles_count, 1);

    let ping_roles_count = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category.id))
        .count(db)
        .await?;
    assert_eq!(ping_roles_count, 2);

    let channels_count = entity::prelude::FleetCategoryChannel::find()
        .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(category.id))
        .count(db)
        .await?;
    assert_eq!(channels_count, 3);

    Ok(())
}

/// Tests creating a category with None duration values.
///
/// Verifies that the repository correctly handles None values for optional
/// duration fields (ping_lead_time, ping_reminder, max_pre_ping).
///
/// Expected: Ok with category created with None durations
#[tokio::test]
async fn creates_category_with_none_durations() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);
    let result = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "No Durations".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await;

    assert!(result.is_ok());
    let category = result.unwrap();
    assert!(category.ping_lead_time.is_none());
    assert!(category.ping_reminder.is_none());
    assert!(category.max_pre_ping.is_none());

    // Verify in database
    let db_category = entity::prelude::FleetCategory::find_by_id(category.id)
        .one(db)
        .await?
        .unwrap();
    assert!(db_category.ping_cooldown.is_none());
    assert!(db_category.ping_reminder.is_none());
    assert!(db_category.max_pre_ping.is_none());

    Ok(())
}

/// Tests creating multiple categories for the same guild.
///
/// Verifies that multiple categories can be created for a single guild
/// and they are properly isolated from each other.
///
/// Expected: Ok with both categories created independently
#[tokio::test]
async fn creates_multiple_categories_for_same_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create guild roles first
    create_guild_role(db, &guild.guild_id, "2001").await?;
    create_guild_role(db, &guild.guild_id, "2002").await?;

    let repo = FleetCategoryRepository::new(db);

    let category1 = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 1".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![2001],
            channels: vec![],
        })
        .await?;

    let category2 = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 2".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![2002],
            channels: vec![],
        })
        .await?;

    assert_ne!(category1.id, category2.id);
    assert_eq!(category1.name, "Category 1");
    assert_eq!(category2.name, "Category 2");

    // Verify each has their own ping roles
    let cat1_roles = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category1.id))
        .all(db)
        .await?;
    assert_eq!(cat1_roles.len(), 1);
    assert_eq!(cat1_roles[0].role_id, "2001");

    let cat2_roles = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category2.id))
        .all(db)
        .await?;
    assert_eq!(cat2_roles.len(), 1);
    assert_eq!(cat2_roles[0].role_id, "2002");

    Ok(())
}
