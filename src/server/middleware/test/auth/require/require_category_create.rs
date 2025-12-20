use super::*;

/// Tests successful permission check for admin user.
///
/// Verifies that an admin user bypasses category-level permission checks
/// and is granted access regardless of their role permissions.
///
/// Expected: Ok(User) with admin privileges
#[tokio::test]
async fn allows_admin_user() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create admin user
    let admin_user = factory::user::UserFactory::new(db)
        .discord_id("123456789")
        .admin(true)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session
        .set_user_id(admin_user.discord_id.clone())
        .await?;

    // Create category (admin doesn't need permission)
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    // Check permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[Permission::CategoryCreate(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.admin);
    assert_eq!(user.discord_id, admin_user.discord_id);

    Ok(())
}

/// Tests successful permission check for user with create role.
///
/// Verifies that a non-admin user with a role that has create permission
/// for the category is granted access.
///
/// Expected: Ok(User)
#[tokio::test]
async fn allows_user_with_create_permission() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::create_user_with_id(db, "123456789").await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Create guild, role, and category
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "987654321").await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create category with create permission for the role
    use crate::server::data::category::FleetCategoryRepository;
    use crate::server::model::category::{AccessRoleData, CreateFleetCategoryParams};

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

    // Check permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[Permission::CategoryCreate(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert!(!returned_user.admin);
    assert_eq!(returned_user.discord_id, user.discord_id);

    Ok(())
}

/// Tests permission denial for user without create permission.
///
/// Verifies that a non-admin user without create permission for the
/// category is denied access.
///
/// Expected: Err(AuthError::AccessDenied)
#[tokio::test]
async fn denies_user_without_create_permission() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::create_user_with_id(db, "123456789").await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Create guild, role, and category
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "111111111").await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create category with only view permission for the role (no create)
    use crate::server::data::category::FleetCategoryRepository;
    use crate::server::model::category::{AccessRoleData, CreateFleetCategoryParams};

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
                can_create: false, // No create permission
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

    // Check permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[Permission::CategoryCreate(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
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

/// Tests permission denial for user with no roles.
///
/// Verifies that a non-admin user without any guild roles is denied
/// access to category creation.
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

    // Create non-admin user
    let user = factory::user::create_user_with_id(db, "123456789").await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Create category (user has no roles assigned)
    let guild = factory::discord_guild::create_guild(db).await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;
    let category =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    // Check permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[Permission::CategoryCreate(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
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

/// Tests user with manage permission also has create access.
///
/// Verifies that manage permission implicitly grants create permission
/// based on the repository logic.
///
/// Expected: Ok(User)
#[tokio::test]
async fn allows_user_with_manage_permission() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::create_user_with_id(db, "123456789").await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Create guild, role, and category
    let guild = factory::discord_guild::create_guild(db).await?;
    let role =
        factory::discord_guild_role::create_guild_role(db, &guild.guild_id, "222222222").await?;
    let ping_format = factory::ping_format::create_ping_format(db, &guild.guild_id).await?;

    // Create category with manage permission (which includes create)
    use crate::server::data::category::FleetCategoryRepository;
    use crate::server::model::category::{AccessRoleData, CreateFleetCategoryParams};

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
                can_manage: true, // Manage permission includes create
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

    // Check permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard
        .require(&[Permission::CategoryCreate(
            guild.guild_id.parse().unwrap(),
            category.id,
        )])
        .await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert!(!returned_user.admin);
    assert_eq!(returned_user.discord_id, user.discord_id);

    Ok(())
}
