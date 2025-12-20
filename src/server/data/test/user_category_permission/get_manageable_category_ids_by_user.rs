use crate::server::data::user_category_permission::UserCategoryPermissionRepository;
use crate::server::model::category::{AccessRoleData, CreateFleetCategoryParams};
use sea_orm::DbErr;
use test_utils::{builder::TestBuilder, factory};

use crate::server::data::category::FleetCategoryRepository;

/// Tests getting manageable category IDs when user has manage permission.
///
/// Verifies that the repository returns category IDs where the user has a role
/// with can_manage permission set to true.
///
/// Expected: Ok with category ID in list
#[tokio::test]
async fn returns_category_ids_with_manage_permission() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create a guild role and assign it to the user
    factory::create_guild_role(db, &guild.guild_id, "1001").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1001).await?;

    // Create category with access role that has can_manage permission
    let category_repo = FleetCategoryRepository::new(db);
    let category = category_repo
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
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    assert_eq!(category_ids.len(), 1);
    assert_eq!(category_ids[0], category.id);

    Ok(())
}

/// Tests empty result when user has no manage permission.
///
/// Verifies that the repository returns an empty list when the user has roles
/// but none have can_manage permission.
///
/// Expected: Ok with empty vec
#[tokio::test]
async fn returns_empty_when_user_lacks_manage_permission() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create a guild role and assign it to the user
    factory::create_guild_role(db, &guild.guild_id, "1001").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1001).await?;

    // Create category with access role that has can_manage = false
    let category_repo = FleetCategoryRepository::new(db);
    category_repo
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
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    assert_eq!(category_ids.len(), 0);

    Ok(())
}

/// Tests empty result when user has no roles.
///
/// Verifies that the repository returns an empty list when the user has no
/// guild roles assigned.
///
/// Expected: Ok with empty vec
#[tokio::test]
async fn returns_empty_when_user_has_no_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create a guild role but don't assign it to the user
    factory::create_guild_role(db, &guild.guild_id, "1001").await?;

    // Create category
    let category_repo = FleetCategoryRepository::new(db);
    category_repo
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
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    assert_eq!(category_ids.len(), 0);

    Ok(())
}

/// Tests multiple manageable categories.
///
/// Verifies that the repository returns IDs for all categories where the user
/// has manage permission.
///
/// Expected: Ok with multiple category IDs
#[tokio::test]
async fn returns_multiple_manageable_category_ids() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create a guild role and assign it to the user
    factory::create_guild_role(db, &guild.guild_id, "1001").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1001).await?;

    let category_repo = FleetCategoryRepository::new(db);

    // Category 1 with manage permission
    let category1 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 1".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Category 2 with manage permission
    let category2 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 2".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Category 3 without manage permission (should be excluded)
    category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 3".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    assert_eq!(category_ids.len(), 2);
    assert!(category_ids.contains(&category1.id));
    assert!(category_ids.contains(&category2.id));

    Ok(())
}

/// Tests filtering by guild ID.
///
/// Verifies that the repository only returns category IDs for the specified guild,
/// even if the user has permissions for categories in other guilds.
///
/// Expected: Ok with only category IDs from specified guild
#[tokio::test]
async fn filters_category_ids_by_guild_id() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild1 = factory::discord_guild::create_guild(db).await?;
    let guild2 = factory::discord_guild::DiscordGuildFactory::new(db)
        .guild_id("999999999")
        .build()
        .await?;

    let ping_format1 = factory::ping_format::create_ping_format(db, &guild1.guild_id).await?;
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild2.guild_id).await?;

    // Create guild role in first guild and assign to user
    factory::create_guild_role(db, &guild1.guild_id, "1001").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1001).await?;

    // Create guild role in second guild and assign to user
    factory::create_guild_role(db, &guild2.guild_id, "2001").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 2001).await?;

    let category_repo = FleetCategoryRepository::new(db);

    // Category in guild 1
    let category1 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild1.guild_id.parse().unwrap(),
            ping_format_id: ping_format1.id,
            name: "Guild 1 Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Category in guild 2 (should be excluded)
    category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild2.guild_id.parse().unwrap(),
            ping_format_id: ping_format2.id,
            name: "Guild 2 Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 2001,
                can_view: true,
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild1.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    assert_eq!(category_ids.len(), 1);
    assert_eq!(category_ids[0], category1.id);

    Ok(())
}

/// Tests user with multiple roles having manage access.
///
/// Verifies that the repository returns category IDs when the user has multiple
/// roles and at least one has manage permission.
///
/// Expected: Ok with category IDs accessible via any role
#[tokio::test]
async fn returns_category_ids_when_user_has_multiple_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create two guild roles and assign both to the user
    factory::create_guild_role(db, &guild.guild_id, "1001").await?;
    factory::create_guild_role(db, &guild.guild_id, "1002").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1001).await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1002).await?;

    let category_repo = FleetCategoryRepository::new(db);

    // Category accessible via first role
    let category1 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 1".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Category accessible via second role
    let category2 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 2".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1002,
                can_view: true,
                can_create: false,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    assert_eq!(category_ids.len(), 2);
    assert!(category_ids.contains(&category1.id));
    assert!(category_ids.contains(&category2.id));

    Ok(())
}

/// Tests no duplicate IDs when multiple roles grant access.
///
/// Verifies that the repository returns each category ID only once, even if
/// multiple user roles grant access to the same category.
///
/// Expected: Ok with unique category IDs
#[tokio::test]
async fn returns_unique_category_ids_with_multiple_access_roles() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let user = factory::user::create_user(db).await?;
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create two guild roles and assign both to the user
    factory::create_guild_role(db, &guild.guild_id, "1001").await?;
    factory::create_guild_role(db, &guild.guild_id, "1002").await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1001).await?;
    factory::create_user_guild_role(db, user.discord_id.parse().unwrap(), 1002).await?;

    let category_repo = FleetCategoryRepository::new(db);

    // Category accessible via both roles
    let category = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Shared Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![
                AccessRoleData {
                    role_id: 1001,
                    can_view: true,
                    can_create: false,
                    can_manage: true,
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

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_category_ids_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let category_ids = result.unwrap();
    // Should only return the category ID once, not twice
    assert_eq!(category_ids.len(), 1);
    assert_eq!(category_ids[0], category.id);

    Ok(())
}
