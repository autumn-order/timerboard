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

/// Tests updating a category's basic fields.
///
/// Verifies that the repository successfully updates a category's name,
/// ping_format_id, and duration fields.
///
/// Expected: Ok with updated category
#[tokio::test]
async fn updates_category_basic_fields() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format1 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format1.id,
            name: "Original Name".to_string(),
            ping_lead_time: Some(Duration::minutes(30)),
            ping_reminder: Some(Duration::minutes(15)),
            max_pre_ping: Some(Duration::hours(2)),
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let result = repo
        .update(UpdateFleetCategoryParams {
            id: created.id,
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format2.id,
            name: "Updated Name".to_string(),
            ping_lead_time: Some(Duration::minutes(45)),
            ping_reminder: Some(Duration::minutes(20)),
            max_pre_ping: Some(Duration::hours(3)),
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.ping_format_id, ping_format2.id);
    assert_eq!(updated.ping_lead_time, Some(Duration::minutes(45)));
    assert_eq!(updated.ping_reminder, Some(Duration::minutes(20)));
    assert_eq!(updated.max_pre_ping, Some(Duration::hours(3)));

    // Verify in database
    let db_category = entity::prelude::FleetCategory::find_by_id(created.id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(db_category.name, "Updated Name");
    assert_eq!(db_category.ping_format_id, ping_format2.id);

    Ok(())
}

/// Tests updating category to clear duration fields.
///
/// Verifies that the repository can update duration fields from Some to None.
///
/// Expected: Ok with durations cleared
#[tokio::test]
async fn updates_category_to_clear_durations() -> Result<(), DbErr> {
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
            ping_lead_time: Some(Duration::minutes(30)),
            ping_reminder: Some(Duration::minutes(15)),
            max_pre_ping: Some(Duration::hours(2)),
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let result = repo
        .update(UpdateFleetCategoryParams {
            id: created.id,
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
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert!(updated.ping_lead_time.is_none());
    assert!(updated.ping_reminder.is_none());
    assert!(updated.max_pre_ping.is_none());

    Ok(())
}

/// Tests updating category replaces access roles.
///
/// Verifies that updating a category completely replaces existing access
/// roles with the new ones provided.
///
/// Expected: Ok with old roles deleted and new roles created
#[tokio::test]
async fn updates_category_replaces_access_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create discord guild roles for foreign key constraints
    create_guild_role(db, &guild.guild_id, "1001").await?;
    create_guild_role(db, &guild.guild_id, "1002").await?;
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
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Update with different access roles
    repo.update(UpdateFleetCategoryParams {
        id: created.id,
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Test Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![
            AccessRoleData {
                role_id: 2001,
                can_view: true,
                can_create: true,
                can_manage: true,
            },
            AccessRoleData {
                role_id: 2002,
                can_view: true,
                can_create: false,
                can_manage: true,
            },
            AccessRoleData {
                role_id: 2003,
                can_view: false,
                can_create: false,
                can_manage: false,
            },
        ],
        ping_roles: vec![],
        channels: vec![],
    })
    .await?;

    // Verify old roles deleted and new roles exist
    let db_access_roles = entity::prelude::FleetCategoryAccessRole::find()
        .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(created.id))
        .all(db)
        .await?;

    assert_eq!(db_access_roles.len(), 3);

    // Old roles should not exist
    assert!(!db_access_roles.iter().any(|r| r.role_id == "1001"));
    assert!(!db_access_roles.iter().any(|r| r.role_id == "1002"));

    // New roles should exist
    assert!(db_access_roles.iter().any(|r| r.role_id == "2001"));
    assert!(db_access_roles.iter().any(|r| r.role_id == "2002"));
    assert!(db_access_roles.iter().any(|r| r.role_id == "2003"));

    Ok(())
}

/// Tests updating category replaces ping roles.
///
/// Verifies that updating a category completely replaces existing ping
/// roles with the new ones provided.
///
/// Expected: Ok with old roles deleted and new roles created
#[tokio::test]
async fn updates_category_replaces_ping_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create discord guild roles for foreign key constraints
    create_guild_role(db, &guild.guild_id, "3001").await?;
    create_guild_role(db, &guild.guild_id, "3002").await?;
    create_guild_role(db, &guild.guild_id, "3003").await?;
    create_guild_role(db, &guild.guild_id, "4001").await?;
    create_guild_role(db, &guild.guild_id, "4002").await?;

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
            ping_roles: vec![3001, 3002, 3003],
            channels: vec![],
        })
        .await?;

    // Update with different ping roles
    repo.update(UpdateFleetCategoryParams {
        id: created.id,
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Test Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![4001, 4002],
        channels: vec![],
    })
    .await?;

    // Verify old roles deleted and new roles exist
    let db_ping_roles = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(created.id))
        .all(db)
        .await?;

    assert_eq!(db_ping_roles.len(), 2);
    let role_ids: Vec<String> = db_ping_roles.iter().map(|r| r.role_id.clone()).collect();

    // Old roles should not exist
    assert!(!role_ids.contains(&"3001".to_string()));
    assert!(!role_ids.contains(&"3002".to_string()));
    assert!(!role_ids.contains(&"3003".to_string()));

    // New roles should exist
    assert!(role_ids.contains(&"4001".to_string()));
    assert!(role_ids.contains(&"4002".to_string()));

    Ok(())
}

/// Tests updating category replaces channels.
///
/// Verifies that updating a category completely replaces existing channels
/// with the new ones provided.
///
/// Expected: Ok with old channels deleted and new channels created
#[tokio::test]
async fn updates_category_replaces_channels() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create discord guild channels for foreign key constraints
    create_guild_channel(db, &guild.guild_id, "5001").await?;
    create_guild_channel(db, &guild.guild_id, "5002").await?;
    create_guild_channel(db, &guild.guild_id, "6001").await?;
    create_guild_channel(db, &guild.guild_id, "6002").await?;
    create_guild_channel(db, &guild.guild_id, "6003").await?;
    create_guild_channel(db, &guild.guild_id, "6004").await?;

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
            channels: vec![5001, 5002],
        })
        .await?;

    // Update with different channels
    repo.update(UpdateFleetCategoryParams {
        id: created.id,
        guild_id: guild.guild_id.parse().unwrap(),
        ping_format_id: ping_format.id,
        name: "Test Category".to_string(),
        ping_lead_time: None,
        ping_reminder: None,
        max_pre_ping: None,
        access_roles: vec![],
        ping_roles: vec![],
        channels: vec![6001, 6002, 6003, 6004],
    })
    .await?;

    // Verify old channels deleted and new channels exist
    let db_channels = entity::prelude::FleetCategoryChannel::find()
        .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(created.id))
        .all(db)
        .await?;

    assert_eq!(db_channels.len(), 4);
    let channel_ids: Vec<String> = db_channels.iter().map(|c| c.channel_id.clone()).collect();

    // Old channels should not exist
    assert!(!channel_ids.contains(&"5001".to_string()));
    assert!(!channel_ids.contains(&"5002".to_string()));

    // New channels should exist
    assert!(channel_ids.contains(&"6001".to_string()));
    assert!(channel_ids.contains(&"6002".to_string()));
    assert!(channel_ids.contains(&"6003".to_string()));
    assert!(channel_ids.contains(&"6004".to_string()));

    Ok(())
}

/// Tests updating category to remove all related entities.
///
/// Verifies that updating a category with empty vectors removes all
/// existing access roles, ping roles, and channels.
///
/// Expected: Ok with all related entities deleted
#[tokio::test]
async fn updates_category_to_remove_all_related_entities() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create discord guild roles and channels for foreign key constraints
    create_guild_role(db, &guild.guild_id, "1001").await?;
    create_guild_role(db, &guild.guild_id, "2001").await?;
    create_guild_role(db, &guild.guild_id, "2002").await?;
    create_guild_channel(db, &guild.guild_id, "3001").await?;

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: true,
                can_manage: true,
            }],
            ping_roles: vec![2001, 2002],
            channels: vec![3001],
        })
        .await?;

    // Update with empty vectors to remove all related entities
    repo.update(UpdateFleetCategoryParams {
        id: created.id,
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

    // Verify all related entities were deleted
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

/// Tests updating a nonexistent category.
///
/// Verifies that attempting to update a category that doesn't exist
/// returns a RecordNotFound error.
///
/// Expected: Err(RecordNotFound)
#[tokio::test]
async fn fails_to_update_nonexistent_category() -> Result<(), DbErr> {
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
        .update(UpdateFleetCategoryParams {
            id: 99999,
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Nonexistent".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DbErr::RecordNotFound(_) => (),
        e => panic!("Expected RecordNotFound error, got {:?}", e),
    }

    Ok(())
}

/// Tests updating category with all fields changed.
///
/// Verifies that all fields can be updated simultaneously in a single
/// update operation.
///
/// Expected: Ok with all fields updated
#[tokio::test]
async fn updates_all_category_fields_at_once() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format1 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create discord guild roles and channels for foreign key constraints
    create_guild_role(db, &guild.guild_id, "1001").await?;
    create_guild_role(db, &guild.guild_id, "1002").await?;
    create_guild_role(db, &guild.guild_id, "1003").await?;
    create_guild_role(db, &guild.guild_id, "2001").await?;
    create_guild_role(db, &guild.guild_id, "2002").await?;
    create_guild_role(db, &guild.guild_id, "2003").await?;
    create_guild_role(db, &guild.guild_id, "2004").await?;
    create_guild_channel(db, &guild.guild_id, "3001").await?;
    create_guild_channel(db, &guild.guild_id, "3002").await?;
    create_guild_channel(db, &guild.guild_id, "3003").await?;

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format1.id,
            name: "Original".to_string(),
            ping_lead_time: Some(Duration::minutes(30)),
            ping_reminder: Some(Duration::minutes(15)),
            max_pre_ping: Some(Duration::hours(2)),
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![2001],
            channels: vec![3001],
        })
        .await?;

    let result = repo
        .update(UpdateFleetCategoryParams {
            id: created.id,
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format2.id,
            name: "Updated".to_string(),
            ping_lead_time: Some(Duration::minutes(60)),
            ping_reminder: Some(Duration::minutes(30)),
            max_pre_ping: Some(Duration::hours(4)),
            access_roles: vec![
                AccessRoleData {
                    role_id: 1002,
                    can_view: true,
                    can_create: true,
                    can_manage: false,
                },
                AccessRoleData {
                    role_id: 1003,
                    can_view: true,
                    can_create: false,
                    can_manage: true,
                },
            ],
            ping_roles: vec![2002, 2003, 2004],
            channels: vec![3002, 3003],
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();

    // Verify basic fields
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.ping_format_id, ping_format2.id);
    assert_eq!(updated.ping_lead_time, Some(Duration::minutes(60)));
    assert_eq!(updated.ping_reminder, Some(Duration::minutes(30)));
    assert_eq!(updated.max_pre_ping, Some(Duration::hours(4)));

    // Verify counts from database
    let access_roles_count = entity::prelude::FleetCategoryAccessRole::find()
        .filter(entity::fleet_category_access_role::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(access_roles_count, 2);

    let ping_roles_count = entity::prelude::FleetCategoryPingRole::find()
        .filter(entity::fleet_category_ping_role::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(ping_roles_count, 3);

    let channels_count = entity::prelude::FleetCategoryChannel::find()
        .filter(entity::fleet_category_channel::Column::FleetCategoryId.eq(created.id))
        .count(db)
        .await?;
    assert_eq!(channels_count, 2);

    Ok(())
}
