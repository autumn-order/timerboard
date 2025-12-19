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

/// Tests deleting a category by ID.
///
/// Verifies that the repository successfully deletes a category
/// from the database.
///
/// Expected: Ok with category deleted
#[tokio::test]
async fn deletes_category_successfully() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
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

    let result = repo.delete(created.id).await;

    assert!(result.is_ok());

    // Verify category no longer exists
    let db_category = entity::prelude::FleetCategory::find_by_id(created.id)
        .one(db)
        .await?;
    assert!(db_category.is_none());

    Ok(())
}

/// Tests deleting category cascades to access roles.
///
/// Verifies that deleting a category also removes all associated
/// access roles through cascade deletion.
///
/// Expected: Ok with category and access roles deleted
#[tokio::test]
async fn deletes_category_cascades_to_access_roles() -> Result<(), DbErr> {
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

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
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
                    can_create: true,
                    can_manage: false,
                },
                AccessRoleData {
                    role_id: 1002,
                    can_view: true,
                    can_create: false,
                    can_manage: true,
                },
            ],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    repo.delete(created.id).await?;

    // Verify access roles are also deleted
    let access_roles_count = entity::prelude::FleetCategoryAccessRole::find()
        .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(access_roles_count, 0);

    Ok(())
}

/// Tests deleting category cascades to ping roles.
///
/// Verifies that deleting a category also removes all associated
/// ping roles through cascade deletion.
///
/// Expected: Ok with category and ping roles deleted
#[tokio::test]
async fn deletes_category_cascades_to_ping_roles() -> Result<(), DbErr> {
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
    let created = repo
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
        .await?;

    repo.delete(created.id).await?;

    // Verify ping roles are also deleted
    let ping_roles_count = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(ping_roles_count, 0);

    Ok(())
}

/// Tests deleting category cascades to channels.
///
/// Verifies that deleting a category also removes all associated
/// channels through cascade deletion.
///
/// Expected: Ok with category and channels deleted
#[tokio::test]
async fn deletes_category_cascades_to_channels() -> Result<(), DbErr> {
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
    create_guild_channel(db, &guild.guild_id, "3003").await?;
    create_guild_channel(db, &guild.guild_id, "3004").await?;

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![3001, 3002, 3003, 3004],
        })
        .await?;

    repo.delete(created.id).await?;

    // Verify channels are also deleted
    let channels_count = entity::prelude::FleetCategoryChannel::find()
        .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(channels_count, 0);

    Ok(())
}

/// Tests deleting category with all related entities.
///
/// Verifies that deleting a category removes the category and all
/// associated access roles, ping roles, and channels.
///
/// Expected: Ok with everything deleted
#[tokio::test]
async fn deletes_category_with_all_related_entities() -> Result<(), DbErr> {
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
    create_guild_channel(db, &guild.guild_id, "3001").await?;
    create_guild_channel(db, &guild.guild_id, "3002").await?;
    create_guild_channel(db, &guild.guild_id, "3003").await?;

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Full Category".to_string(),
            ping_lead_time: Some(Duration::minutes(30)),
            ping_reminder: Some(Duration::minutes(15)),
            max_pre_ping: Some(Duration::hours(2)),
            access_roles: vec![
                AccessRoleData {
                    role_id: 1001,
                    can_view: true,
                    can_create: true,
                    can_manage: true,
                },
                AccessRoleData {
                    role_id: 1002,
                    can_view: true,
                    can_create: false,
                    can_manage: false,
                },
            ],
            ping_roles: vec![2001, 2002],
            channels: vec![3001, 3002, 3003],
        })
        .await?;

    repo.delete(created.id).await?;

    // Verify category deleted
    let db_category = entity::prelude::FleetCategory::find_by_id(created.id)
        .one(db)
        .await?;
    assert!(db_category.is_none());

    // Verify all related entities deleted
    let access_roles_count = entity::prelude::FleetCategoryAccessRole::find()
        .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(access_roles_count, 0);

    let ping_roles_count = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(ping_roles_count, 0);

    let channels_count = entity::prelude::FleetCategoryChannel::find()
        .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(channels_count, 0);

    Ok(())
}

/// Tests deleting nonexistent category succeeds silently.
///
/// Verifies that attempting to delete a category that doesn't exist
/// completes without error (delete is idempotent).
///
/// Expected: Ok (no error)
#[tokio::test]
async fn deletes_nonexistent_category_succeeds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = FleetCategoryRepository::new(db);
    let result = repo.delete(99999).await;

    assert!(result.is_ok());

    Ok(())
}

/// Tests deleting one category doesn't affect others.
///
/// Verifies that deleting a specific category only removes that category
/// and its related entities, leaving other categories untouched.
///
/// Expected: Ok with only target category deleted
#[tokio::test]
async fn deletes_category_without_affecting_others() -> Result<(), DbErr> {
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

    // Delete category1
    repo.delete(category1.id).await?;

    // Verify category1 deleted
    let db_category1 = entity::prelude::FleetCategory::find_by_id(category1.id)
        .one(db)
        .await?;
    assert!(db_category1.is_none());

    let cat1_roles = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category1.id))
        .count(db)
        .await?;
    assert_eq!(cat1_roles, 0);

    // Verify category2 still exists
    let db_category2 = entity::prelude::FleetCategory::find_by_id(category2.id)
        .one(db)
        .await?;
    assert!(db_category2.is_some());

    let cat2_roles = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(category2.id))
        .count(db)
        .await?;
    assert_eq!(cat2_roles, 1);

    Ok(())
}
