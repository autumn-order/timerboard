use super::*;

/// Tests that admin user bypasses category view permission checks.
///
/// Verifies that an admin user can view any category regardless of whether
/// they have the specific view permission for that category.
///
/// Expected: Ok(User) with admin user
#[tokio::test]
async fn admin_bypasses_view_check() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create admin user
    let admin = factory::user::UserFactory::new(db)
        .discord_id("123456789")
        .admin(true)
        .build()
        .await?;

    // Create guild and category
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(admin.discord_id.clone()).await?;

    // Create auth guard and check permission
    let guard = AuthGuard::new(db, session);
    let result = guard
        .require(&[Permission::CategoryView(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.discord_id, admin.discord_id);
    assert!(user.admin);

    Ok(())
}

/// Tests that user with view permission can access category.
///
/// Verifies that a non-admin user with the correct role permission
/// can successfully view a category.
///
/// Expected: Ok(User)
#[tokio::test]
async fn allows_user_with_view_permission() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("987654321")
        .admin(false)
        .build()
        .await?;

    // Create guild, role, and category with view permission
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "987654321").await?;

    // Assign user to role
    factory::user_discord_guild_role::create_user_guild_role(
        db,
        user.discord_id.parse().unwrap(),
        role.role_id.parse().unwrap(),
    )
    .await?;

    // Create category with view access for this role
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
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
                role_id: role.role_id.parse().unwrap(),
                can_view: true,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Check permission
    let guard = AuthGuard::new(db, session);
    let result = guard
        .require(&[Permission::CategoryView(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert_eq!(returned_user.discord_id, user.discord_id);
    assert!(!returned_user.admin);

    Ok(())
}

/// Tests that user without view permission is denied access.
///
/// Verifies that a non-admin user without the correct role permission
/// is denied access to view a category.
///
/// Expected: Err(AuthError::AccessDenied)
#[tokio::test]
async fn denies_user_without_view_permission() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("987654321")
        .admin(false)
        .build()
        .await?;

    // Create guild, role, and category WITHOUT view permission
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "111111111").await?;

    // Assign user to role
    factory::user_discord_guild_role::create_user_guild_role(
        db,
        user.discord_id.parse().unwrap(),
        role.role_id.parse().unwrap(),
    )
    .await?;

    // Create category with NO view access for this role
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
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
                role_id: role.role_id.parse().unwrap(),
                can_view: false,
                can_create: false,
                can_manage: false,
            }],
            ping_roles: vec![],
            channels: vec![],
        })
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Check permission
    let guard = AuthGuard::new(db, session);
    let result = guard
        .require(&[Permission::CategoryView(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::AuthErr(AuthError::AccessDenied(user_id, msg)) => {
            assert_eq!(user_id, user.discord_id.parse::<u64>().unwrap());
            assert!(msg.contains("does not have view access"));
        }
        _ => panic!("Expected AuthError::AccessDenied"),
    }

    Ok(())
}

/// Tests that user with no roles is denied access.
///
/// Verifies that a user without any guild roles is denied access
/// to view a category.
///
/// Expected: Err(AuthError::AccessDenied)
#[tokio::test]
async fn denies_user_with_no_roles() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user (no roles assigned)
    let user = factory::user::UserFactory::new(db)
        .discord_id("987654321")
        .admin(false)
        .build()
        .await?;

    // Create guild and category
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Check permission
    let guard = AuthGuard::new(db, session);
    let result = guard
        .require(&[Permission::CategoryView(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::AuthErr(AuthError::AccessDenied(user_id, msg)) => {
            assert_eq!(user_id, user.discord_id.parse::<u64>().unwrap());
            assert!(msg.contains("does not have view access"));
        }
        _ => panic!("Expected AuthError::AccessDenied"),
    }

    Ok(())
}

/// Tests that user in wrong guild is denied access.
///
/// Verifies that a user with permissions in one guild cannot access
/// categories in a different guild.
///
/// Expected: Err(AuthError::AccessDenied)
#[tokio::test]
async fn denies_user_from_different_guild() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("987654321")
        .admin(false)
        .build()
        .await?;

    // Create two different guilds
    let guild1 = factory::discord_guild::create_guild(db).await?;
    let guild2 = factory::discord_guild::create_guild(db).await?;

    // Create role in guild1 and assign user to it
    let role1 =
        factory::discord_guild_role::create_guild_role(db, &guild1.guild_id, "333333333").await?;
    factory::user_discord_guild_role::create_user_guild_role(
        db,
        user.discord_id.parse().unwrap(),
        role1.role_id.parse().unwrap(),
    )
    .await?;

    // Create category in guild2 (different guild)
    let ping_format2 = factory::ping_format::create_ping_format(db, &guild2.guild_id).await?;
    let category2 =
        factory::fleet_category::create_category(db, &guild2.guild_id, ping_format2.id).await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Try to check permission for guild2's category
    let guard = AuthGuard::new(db, session);
    let result = guard
        .require(&[Permission::CategoryView(
            guild2.guild_id.parse().unwrap(),
            category2.id,
        )])
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::AuthErr(AuthError::AccessDenied(user_id, msg)) => {
            assert_eq!(user_id, user.discord_id.parse::<u64>().unwrap());
            assert!(msg.contains("does not have view access"));
        }
        _ => panic!("Expected AuthError::AccessDenied"),
    }

    Ok(())
}
