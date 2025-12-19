use super::*;

/// Helper function to create a Discord guild role for testing
async fn create_guild_role(
    db: &DatabaseConnection,
    guild_id: &str,
    role_id: &str,
    position: i16,
) -> Result<(), DbErr> {
    entity::discord_guild_role::ActiveModel {
        guild_id: ActiveValue::Set(guild_id.to_string()),
        role_id: ActiveValue::Set(role_id.to_string()),
        name: ActiveValue::Set(format!("Role {}", role_id)),
        color: ActiveValue::Set(String::new()),
        position: ActiveValue::Set(position),
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
    position: i32,
) -> Result<(), DbErr> {
    entity::discord_guild_channel::ActiveModel {
        guild_id: ActiveValue::Set(guild_id.to_string()),
        channel_id: ActiveValue::Set(channel_id.to_string()),
        name: ActiveValue::Set(format!("Channel {}", channel_id)),
        position: ActiveValue::Set(position),
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Tests getting a category by ID with all related entities.
///
/// Verifies that the repository successfully retrieves a category with
/// its ping format, access roles, ping roles, and channels.
///
/// Expected: Ok(Some(FleetCategoryWithRelations))
#[tokio::test]
async fn gets_category_with_all_relations() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create discord guild roles for foreign key constraints
    create_guild_role(db, &guild.guild_id, "1001", 0).await?;
    create_guild_role(db, &guild.guild_id, "2001", 0).await?;
    create_guild_role(db, &guild.guild_id, "2002", 0).await?;

    // Create discord guild channel for foreign key constraint
    create_guild_channel(db, &guild.guild_id, "3001", 0).await?;

    let access_role = AccessRoleData {
        role_id: 1001,
        can_view: true,
        can_create: true,
        can_manage: false,
    };

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: Some(Duration::minutes(30)),
            ping_reminder: Some(Duration::minutes(15)),
            max_pre_ping: Some(Duration::hours(2)),
            access_roles: vec![access_role],
            ping_roles: vec![2001, 2002],
            channels: vec![3001],
        })
        .await?;

    let result = repo.get_by_id(created.id).await;

    assert!(result.is_ok());
    let category_with_relations = result.unwrap();
    assert!(category_with_relations.is_some());

    let relations = category_with_relations.unwrap();
    assert_eq!(relations.category.id, created.id);
    assert_eq!(relations.category.name, "Test Category");
    assert!(relations.ping_format.is_some());
    assert_eq!(relations.ping_format.unwrap().id, ping_format.id);
    assert_eq!(relations.access_roles.len(), 1);
    assert_eq!(relations.ping_roles.len(), 2);
    assert_eq!(relations.channels.len(), 1);

    Ok(())
}

/// Tests getting a category by ID without related entities.
///
/// Verifies that the repository successfully retrieves a category that
/// has no access roles, ping roles, or channels.
///
/// Expected: Ok(Some(FleetCategoryWithRelations)) with empty relations
#[tokio::test]
async fn gets_category_without_related_entities() -> Result<(), DbErr> {
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
            name: "Empty Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let result = repo.get_by_id(created.id).await;

    assert!(result.is_ok());
    let category_with_relations = result.unwrap();
    assert!(category_with_relations.is_some());

    let relations = category_with_relations.unwrap();
    assert_eq!(relations.category.id, created.id);
    assert_eq!(relations.access_roles.len(), 0);
    assert_eq!(relations.ping_roles.len(), 0);
    assert_eq!(relations.channels.len(), 0);

    Ok(())
}

/// Tests getting a nonexistent category by ID.
///
/// Verifies that the repository returns None when attempting to retrieve
/// a category that doesn't exist in the database.
///
/// Expected: Ok(None)
#[tokio::test]
async fn returns_none_for_nonexistent_category() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = FleetCategoryRepository::new(db);
    let result = repo.get_by_id(99999).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    Ok(())
}

/// Tests getting a category with enriched role data.
///
/// Verifies that access roles and ping roles are properly enriched with
/// Discord role metadata (name, color, position) when the corresponding
/// guild roles exist.
///
/// Expected: Ok(Some(FleetCategoryWithRelations)) with enriched role data
#[tokio::test]
async fn gets_category_with_enriched_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create Discord guild roles
    create_guild_role(db, &guild.guild_id, "1001", 10).await?;
    create_guild_role(db, &guild.guild_id, "2001", 5).await?;

    let access_role = AccessRoleData {
        role_id: 1001,
        can_view: true,
        can_create: true,
        can_manage: false,
    };

    let repo = FleetCategoryRepository::new(db);
    let created = repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Test Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![access_role],
            ping_roles: vec![2001],
            channels: vec![],
        })
        .await?;

    let result = repo.get_by_id(created.id).await;

    assert!(result.is_ok());
    let relations = result.unwrap().unwrap();

    // Verify access role is enriched
    assert_eq!(relations.access_roles.len(), 1);
    let (access_role_entity, access_role_discord) = &relations.access_roles[0];
    assert_eq!(access_role_entity.role_id, "1001");
    assert!(access_role_discord.is_some());
    let discord_role = access_role_discord.as_ref().unwrap();
    assert_eq!(discord_role.name, "Role 1001");
    assert_eq!(discord_role.position, 10);

    // Verify ping role is enriched
    assert_eq!(relations.ping_roles.len(), 1);
    let (ping_role_entity, ping_role_discord) = &relations.ping_roles[0];
    assert_eq!(ping_role_entity.role_id, "2001");
    assert!(ping_role_discord.is_some());
    let ping_discord = ping_role_discord.as_ref().unwrap();
    assert_eq!(ping_discord.name, "Role 2001");

    Ok(())
}

/// Tests getting a category with enriched channel data.
///
/// Verifies that channels are properly enriched with Discord channel
/// metadata (name, position) when the corresponding guild channels exist.
///
/// Expected: Ok(Some(FleetCategoryWithRelations)) with enriched channel data
#[tokio::test]
async fn gets_category_with_enriched_channels() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create Discord guild channels
    create_guild_channel(db, &guild.guild_id, "3001", 1).await?;
    create_guild_channel(db, &guild.guild_id, "3002", 2).await?;

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
            channels: vec![3001, 3002],
        })
        .await?;

    let result = repo.get_by_id(created.id).await;

    assert!(result.is_ok());
    let relations = result.unwrap().unwrap();

    // Verify channels are enriched and sorted by position
    assert_eq!(relations.channels.len(), 2);

    let (channel1_entity, channel1_discord) = &relations.channels[0];
    assert_eq!(channel1_entity.channel_id, "3001");
    assert!(channel1_discord.is_some());
    assert_eq!(channel1_discord.as_ref().unwrap().name, "Channel 3001");
    assert_eq!(channel1_discord.as_ref().unwrap().position, 1);

    let (channel2_entity, channel2_discord) = &relations.channels[1];
    assert_eq!(channel2_entity.channel_id, "3002");
    assert!(channel2_discord.is_some());
    assert_eq!(channel2_discord.as_ref().unwrap().name, "Channel 3002");
    assert_eq!(channel2_discord.as_ref().unwrap().position, 2);

    Ok(())
}

/// Tests role sorting by position.
///
/// Verifies that access roles and ping roles are sorted by position in
/// descending order (higher position = displayed first).
///
/// Expected: Ok(Some(FleetCategoryWithRelations)) with roles sorted correctly
#[tokio::test]
async fn sorts_roles_by_position_descending() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create roles with different positions
    create_guild_role(db, &guild.guild_id, "1001", 1).await?;
    create_guild_role(db, &guild.guild_id, "1002", 10).await?;

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
                    can_create: false,
                    can_manage: false,
                },
            ],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let result = repo.get_by_id(created.id).await;

    assert!(result.is_ok());
    let relations = result.unwrap().unwrap();

    // Verify roles are sorted by position descending (highest first)
    assert_eq!(relations.access_roles.len(), 2);
    let first_role = &relations.access_roles[0];
    let second_role = &relations.access_roles[1];

    assert_eq!(first_role.1.as_ref().unwrap().name, "Role 1002");
    assert_eq!(first_role.1.as_ref().unwrap().position, 10);
    assert_eq!(second_role.1.as_ref().unwrap().name, "Role 1001");
    assert_eq!(second_role.1.as_ref().unwrap().position, 1);

    Ok(())
}

/// Tests channel sorting by position.
///
/// Verifies that channels are sorted by position in ascending order
/// (lower position = displayed first).
///
/// Expected: Ok(Some(FleetCategoryWithRelations)) with channels sorted correctly
#[tokio::test]
async fn sorts_channels_by_position_ascending() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create channels with different positions
    create_guild_channel(db, &guild.guild_id, "3001", 5).await?;
    create_guild_channel(db, &guild.guild_id, "3002", 1).await?;

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
            channels: vec![3001, 3002],
        })
        .await?;

    let result = repo.get_by_id(created.id).await;

    assert!(result.is_ok());
    let relations = result.unwrap().unwrap();

    // Verify channels are sorted by position ascending (lowest first)
    assert_eq!(relations.channels.len(), 2);
    let first_channel = &relations.channels[0];
    let second_channel = &relations.channels[1];

    assert_eq!(first_channel.1.as_ref().unwrap().name, "Channel 3002");
    assert_eq!(first_channel.1.as_ref().unwrap().position, 1);
    assert_eq!(second_channel.1.as_ref().unwrap().name, "Channel 3001");
    assert_eq!(second_channel.1.as_ref().unwrap().position, 5);

    Ok(())
}
