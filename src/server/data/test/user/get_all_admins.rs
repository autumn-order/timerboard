use super::*;

/// Tests retrieving all admin users.
///
/// Verifies that the repository returns all users with admin privileges
/// and excludes regular users.
///
/// Expected: Ok with vector containing only admin users
#[tokio::test]
async fn returns_only_admin_users() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create mix of admin and regular users
    repo.upsert(UpsertUserParam {
        discord_id: "111111111".to_string(),
        name: "Admin1".to_string(),
        is_admin: Some(true),
    })
    .await?;

    repo.upsert(UpsertUserParam {
        discord_id: "222222222".to_string(),
        name: "RegularUser".to_string(),
        is_admin: None,
    })
    .await?;

    repo.upsert(UpsertUserParam {
        discord_id: "333333333".to_string(),
        name: "Admin2".to_string(),
        is_admin: Some(true),
    })
    .await?;

    let result = repo.get_all_admins().await;

    assert!(result.is_ok());
    let admins = result.unwrap();
    assert_eq!(admins.len(), 2);
    assert!(admins.iter().all(|u| u.admin));
    assert_eq!(admins[0].name, "Admin1");
    assert_eq!(admins[1].name, "Admin2");

    Ok(())
}

/// Tests retrieving admins when none exist.
///
/// Verifies that the repository returns an empty vector when no admin
/// users exist in the database.
///
/// Expected: Ok with empty vector
#[tokio::test]
async fn returns_empty_when_no_admins() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create only regular users
    repo.upsert(UpsertUserParam {
        discord_id: "111111111".to_string(),
        name: "RegularUser".to_string(),
        is_admin: None,
    })
    .await?;

    let result = repo.get_all_admins().await;

    assert!(result.is_ok());
    let admins = result.unwrap();
    assert!(admins.is_empty());

    Ok(())
}

/// Tests admins are ordered alphabetically by name.
///
/// Verifies that admin users are returned sorted by name in ascending order.
///
/// Expected: Ok with admins sorted by name
#[tokio::test]
async fn orders_admins_by_name() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create admins in non-alphabetical order
    repo.upsert(UpsertUserParam {
        discord_id: "222222222".to_string(),
        name: "Zara".to_string(),
        is_admin: Some(true),
    })
    .await?;

    repo.upsert(UpsertUserParam {
        discord_id: "111111111".to_string(),
        name: "Alice".to_string(),
        is_admin: Some(true),
    })
    .await?;

    let result = repo.get_all_admins().await;

    assert!(result.is_ok());
    let admins = result.unwrap();
    assert_eq!(admins[0].name, "Alice");
    assert_eq!(admins[1].name, "Zara");

    Ok(())
}
