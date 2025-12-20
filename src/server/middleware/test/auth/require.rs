use super::*;

mod require_admin;
mod require_category_create;
mod require_category_view;

/// Tests multiple permissions are all checked.
///
/// Verifies that when multiple permissions are required, all of them
/// must be satisfied for access to be granted.
///
/// Expected: Ok(User) when all permissions are met
#[tokio::test]
async fn requires_all_permissions() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("123456789")
        .admin(false)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Create guild, role, and two categories
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "123456789").await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create category 1 with view permission
    let category_repo = FleetCategoryRepository::new(db);
    let category1 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 1".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: role.role_id.parse().unwrap(),
                can_view: true,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Create category 2 with create permission
    let category2 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 2".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: role.role_id.parse().unwrap(),
                can_view: true,
                can_create: true,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Assign role to user
    factory::user_discord_guild_role::create_user_guild_role(
        db,
        user.discord_id.parse().unwrap(),
        role.role_id.parse().unwrap(),
    )
    .await?;

    // Check multiple permissions - user has both view for category1 and create for category2
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[
            Permission::CategoryView(guild.guild_id.parse().unwrap(), category1.id),
            Permission::CategoryCreate(guild.guild_id.parse().unwrap(), category2.id),
        ])
        .await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert_eq!(returned_user.discord_id, user.discord_id);

    Ok(())
}

/// Tests that if any permission fails, the whole check fails.
///
/// Verifies that when checking multiple permissions, if the user lacks
/// any one of them, access is denied.
///
/// Expected: Err(AuthError::AccessDenied) for the first failed permission
#[tokio::test]
async fn fails_if_any_permission_missing() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("123456789")
        .admin(false)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Create guild, role, and two categories
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "987654321").await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create category 1 with view permission
    let category_repo = FleetCategoryRepository::new(db);
    let category1 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 1".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: role.role_id.parse().unwrap(),
                can_view: true,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Create category 2 WITHOUT create permission (only view)
    let category2 = category_repo
        .create(CreateFleetCategoryParams {
            guild_id: guild.guild_id.parse().unwrap(),
            ping_format_id: ping_format.id,
            name: "Category 2".to_string(),
            ping_lead_time: None,
            ping_reminder: None,
            max_pre_ping: None,
            access_roles: vec![AccessRoleData {
                role_id: role.role_id.parse().unwrap(),
                can_view: true,
                can_create: false, // User doesn't have create permission here
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Assign role to user
    factory::user_discord_guild_role::create_user_guild_role(
        db,
        user.discord_id.parse().unwrap(),
        role.role_id.parse().unwrap(),
    )
    .await?;

    // Check multiple permissions - user has view for category1 but NOT create for category2
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[
            Permission::CategoryView(guild.guild_id.parse().unwrap(), category1.id),
            Permission::CategoryCreate(guild.guild_id.parse().unwrap(), category2.id),
        ])
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::AuthErr(AuthError::AccessDenied(user_id, msg)) => {
            assert_eq!(user_id, 123456789);
            assert!(msg.contains("create access"));
        }
        e => panic!("Expected AccessDenied error, got: {:?}", e),
    }

    Ok(())
}

/// Tests admin user passes all permission checks.
///
/// Verifies that an admin user passes multiple permission checks
/// without needing the specific role permissions.
///
/// Expected: Ok(User) with admin user
#[tokio::test]
async fn admin_passes_multiple_permissions() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create admin user
    let admin = factory::user::UserFactory::new(db)
        .discord_id("999999999")
        .admin(true)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(admin.discord_id.clone()).await?;

    // Create guild and categories (admin doesn't need role permissions)
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category1 =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;
    let category2 =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    // Check multiple permissions - admin should bypass all checks
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[
            Permission::CategoryView(guild.guild_id.parse().unwrap(), category1.id),
            Permission::CategoryCreate(guild.guild_id.parse().unwrap(), category2.id),
        ])
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.admin);
    assert_eq!(user.discord_id, admin.discord_id);

    Ok(())
}

/// Tests empty permission list grants access.
///
/// Verifies that when no permissions are required, any authenticated
/// user with a valid database record is granted access.
///
/// Expected: Ok(User)
#[tokio::test]
async fn empty_permission_list_grants_access() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create regular user
    let user = factory::user::UserFactory::new(db)
        .discord_id("123456789")
        .admin(false)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Check with empty permissions list
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard.require(&[]).await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert_eq!(returned_user.discord_id, user.discord_id);

    Ok(())
}
