use crate::server::data::user_category_permission::UserCategoryPermissionRepository;
use crate::server::model::category::{AccessRoleData, CreateFleetCategoryParams};
use sea_orm::DbErr;
use test_utils::{builder::TestBuilder, factory};

use crate::server::data::category::FleetCategoryRepository;

/// Tests user can create in category when they have the required role.
///
/// Verifies that the repository returns true when the user has a role
/// with can_create permission for the category.
///
/// Expected: Ok(true)
#[tokio::test]
async fn returns_true_when_user_has_create_permission() -> Result<(), DbErr> {
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
                can_view: false,
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .user_can_create_category(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
            category.id,
        )
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

/// Tests user cannot create in category when they lack the role.
///
/// Verifies that the repository returns false when the user does not have
/// any roles with can_create permission for the category.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_when_user_lacks_create_permission() -> Result<(), DbErr> {
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

    // Create category with access role
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
                can_view: false,
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .user_can_create_category(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
            category.id,
        )
        .await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests user cannot create when role has can_create set to false.
///
/// Verifies that the repository returns false when the user has a role
/// associated with the category but can_create is explicitly false.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_when_role_has_create_disabled() -> Result<(), DbErr> {
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

    // Create category with access role that has can_create = false
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
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .user_can_create_category(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
            category.id,
        )
        .await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests user with no roles returns false.
///
/// Verifies that the repository returns false when the user has no guild
/// roles assigned to them.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_when_user_has_no_roles() -> Result<(), DbErr> {
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

    // Create category with access role
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
                can_view: false,
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    let repo = UserCategoryPermissionRepository::new(db);
    let result = repo
        .user_can_create_category(
            user.discord_id.parse().unwrap(),
            guild.guild_id.parse().unwrap(),
            category.id,
        )
        .await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}
