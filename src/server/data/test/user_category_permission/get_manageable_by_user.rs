use crate::server::data::user_category_permission::UserCategoryPermissionRepository;
use crate::server::model::category::{AccessRoleData, CreateFleetCategoryParams};
use sea_orm::DbErr;
use test_utils::{builder::TestBuilder, factory};

use crate::server::data::category::FleetCategoryRepository;

/// Tests getting manageable categories when user has can_create permission.
///
/// Verifies that the repository returns categories where the user has a role
/// with can_create permission set to true.
///
/// Expected: Ok with category in list
#[tokio::test]
async fn returns_categories_with_create_permission() -> Result<(), DbErr> {
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

    // Create category with access role that has can_create permission
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
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, category.id);
    assert_eq!(categories[0].name, "Test Category");

    Ok(())
}

/// Tests getting manageable categories when user has can_manage permission.
///
/// Verifies that the repository returns categories where the user has a role
/// with can_manage permission set to true.
///
/// Expected: Ok with category in list
#[tokio::test]
async fn returns_categories_with_manage_permission() -> Result<(), DbErr> {
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
            name: "Manageable Category".to_string(),
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
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, category.id);
    assert_eq!(categories[0].name, "Manageable Category");

    Ok(())
}

/// Tests getting manageable categories when user has both permissions.
///
/// Verifies that the repository returns categories where the user has a role
/// with both can_create and can_manage permissions.
///
/// Expected: Ok with category in list
#[tokio::test]
async fn returns_categories_with_both_permissions() -> Result<(), DbErr> {
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

    // Create category with both permissions
    let category_repo = FleetCategoryRepository::new(db);
    let category = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Fully Accessible Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: true,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, category.id);

    Ok(())
}

/// Tests empty result when user has only view permission.
///
/// Verifies that the repository returns an empty list when the user only has
/// can_view permission but not can_create or can_manage.
///
/// Expected: Ok with empty vec
#[tokio::test]
async fn returns_empty_when_user_has_only_view_permission() -> Result<(), DbErr> {
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

    // Create category with only view permission
    let category_repo = FleetCategoryRepository::new(db);
    category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "View Only Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 0);

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
                can_create: true,
                can_manage: true,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 0);

    Ok(())
}

/// Tests multiple categories with mixed permissions.
///
/// Verifies that the repository returns only categories where the user has
/// can_create or can_manage permission, filtering out view-only categories.
///
/// Expected: Ok with 2 categories
#[tokio::test]
async fn returns_only_manageable_categories_from_mixed_set() -> Result<(), DbErr> {
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

    // Category with create permission
    let category1 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Create Category".to_string(),
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

    // Category with manage permission
    let category2 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Manage Category".to_string(),
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

    // Category with only view permission (should be excluded)
    category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "View Only Category".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: 1001,
                can_view: true,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 2);

    // Should be ordered by name
    assert_eq!(categories[0].id, category1.id);
    assert_eq!(categories[0].name, "Create Category");
    assert_eq!(categories[1].id, category2.id);
    assert_eq!(categories[1].name, "Manage Category");

    Ok(())
}

/// Tests user with multiple roles having different permissions.
///
/// Verifies that the repository returns categories when the user has multiple
/// roles and at least one has the required permission.
///
/// Expected: Ok with category in list
#[tokio::test]
async fn returns_categories_when_user_has_multiple_roles() -> Result<(), DbErr> {
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
                can_create: true,
                can_manage: false,
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
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 2);

    let ids: Vec<i32> = categories.iter().map(|c| c.id).collect();
    assert!(ids.contains(&category1.id));
    assert!(ids.contains(&category2.id));

    Ok(())
}

/// Tests filtering by guild ID.
///
/// Verifies that the repository only returns categories for the specified guild,
/// even if the user has permissions for categories in other guilds.
///
/// Expected: Ok with only categories from specified guild
#[tokio::test]
async fn filters_categories_by_guild_id() -> Result<(), DbErr> {
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
                can_create: true,
                can_manage: false,
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
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .get_manageable_by_user(
            user.discord_id.parse().unwrap(),
            guild1.guild_id.parse().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].id, category1.id);
    assert_eq!(categories[0].name, "Guild 1 Category");

    Ok(())
}
