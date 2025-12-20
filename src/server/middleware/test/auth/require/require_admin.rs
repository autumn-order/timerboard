use super::*;

/// Tests admin user successfully passes admin permission check.
///
/// Verifies that the AuthGuard grants access when the user is authenticated,
/// exists in the database, and has admin privileges.
///
/// Expected: Ok(User) with admin=true
#[tokio::test]
async fn grants_access_to_admin_user() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("123456789")
        .name("AdminUser")
        .admin(true)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Check admin permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard.require(&[Permission::Admin]).await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert_eq!(returned_user.discord_id, "123456789");
    assert_eq!(returned_user.name, "AdminUser");
    assert!(returned_user.admin);

    Ok(())
}

/// Tests non-admin user is denied admin permission.
///
/// Verifies that the AuthGuard denies access when the user is authenticated,
/// exists in the database, but lacks admin privileges.
///
/// Expected: Err(AuthError::AccessDenied)
#[tokio::test]
async fn denies_access_to_non_admin_user() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Create non-admin user
    let user = factory::user::UserFactory::new(db)
        .discord_id("987654321")
        .name("RegularUser")
        .admin(false)
        .build()
        .await?;

    // Set user in session
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id(user.discord_id.clone()).await?;

    // Check admin permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard.require(&[Permission::Admin]).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    match error {
        AppError::AuthErr(auth_error) => match auth_error {
            AuthError::AccessDenied(user_id, message) => {
                assert_eq!(user_id, 987654321);
                assert!(message.contains("admin"));
            }
            _ => panic!("Expected AccessDenied error, got: {:?}", auth_error),
        },
        _ => panic!("Expected AuthError, got: {:?}", error),
    }

    Ok(())
}

/// Tests unauthenticated user is denied admin permission.
///
/// Verifies that the AuthGuard denies access when there is no user ID
/// in the session (user not logged in).
///
/// Expected: Err(AuthError::UserNotInSession)
#[tokio::test]
async fn denies_access_when_not_authenticated() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Don't set user in session - simulate unauthenticated request

    // Check admin permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard.require(&[Permission::Admin]).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    match error {
        AppError::AuthErr(auth_error) => match auth_error {
            AuthError::UserNotInSession => {}
            _ => panic!("Expected UserNotInSession error, got: {:?}", auth_error),
        },
        _ => panic!("Expected AuthError, got: {:?}", error),
    }

    Ok(())
}

/// Tests user in session but not in database is denied.
///
/// Verifies that the AuthGuard denies access when the user ID exists in
/// the session but the user record does not exist in the database.
///
/// Expected: Err(AuthError::UserNotInDatabase)
#[tokio::test]
async fn denies_access_when_user_not_in_database() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Set user ID in session without creating user in database
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id("999999999".to_string()).await?;

    // Check admin permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard.require(&[Permission::Admin]).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    match error {
        AppError::AuthErr(auth_error) => match auth_error {
            AuthError::UserNotInDatabase(user_id) => {
                assert_eq!(user_id, 999999999);
            }
            _ => panic!("Expected UserNotInDatabase error, got: {:?}", auth_error),
        },
        _ => panic!("Expected AuthError, got: {:?}", error),
    }

    Ok(())
}

/// Tests invalid user ID format in session is rejected.
///
/// Verifies that the AuthGuard returns an error when the user ID in the
/// session cannot be parsed as a valid u64.
///
/// Expected: Err(AppError::InternalError)
#[tokio::test]
async fn rejects_invalid_user_id_format() -> Result<(), AppError> {
    let mut test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let (db, session) = test.db_and_session().await.unwrap();

    // Set invalid user ID in session (not a valid u64)
    let auth_session = AuthSession::new(session);
    auth_session.set_user_id("not-a-number".to_string()).await?;

    // Check admin permission
    let auth_guard = AuthGuard::new(db, session);
    let result = auth_guard.require(&[Permission::Admin]).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    match error {
        AppError::InternalError(message) => {
            assert!(message.contains("Failed to parse user_id"));
        }
        _ => panic!("Expected InternalError, got: {:?}", error),
    }

    Ok(())
}
