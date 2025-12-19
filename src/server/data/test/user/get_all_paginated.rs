use super::*;

/// Tests pagination with multiple pages.
///
/// Verifies that the repository correctly paginates users and returns
/// the appropriate subset for the requested page along with accurate
/// total count.
///
/// Expected: Ok with correct page of users and total count
#[tokio::test]
async fn returns_correct_page_of_users() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create 5 users
    for i in 1..=5 {
        repo.upsert(UpsertUserParam {
            discord_id: format!("{}", 100000000 + i),
            name: format!("User{}", i),
            is_admin: None,
        })
        .await?;
    }

    // Get first page (2 per page)
    let result = repo.get_all_paginated(0, 2).await;

    assert!(result.is_ok());
    let (users, total) = result.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(total, 3); // Total pages (5 users / 2 per page = 2.5 -> 3 pages)

    // Get second page
    let result = repo.get_all_paginated(1, 2).await;
    assert!(result.is_ok());
    let (users, _) = result.unwrap();
    assert_eq!(users.len(), 2);

    Ok(())
}

/// Tests pagination with empty database.
///
/// Verifies that the repository correctly handles pagination when no users
/// exist in the database.
///
/// Expected: Ok with empty vector and zero total
#[tokio::test]
async fn returns_empty_for_no_users() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);
    let result = repo.get_all_paginated(0, 10).await;

    assert!(result.is_ok());
    let (users, total) = result.unwrap();
    assert!(users.is_empty());
    assert_eq!(total, 0);

    Ok(())
}

/// Tests users are ordered alphabetically by name.
///
/// Verifies that the repository returns users sorted by name in ascending
/// order regardless of creation order.
///
/// Expected: Ok with users sorted by name
#[tokio::test]
async fn orders_users_by_name() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(entity::prelude::User)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = UserRepository::new(db);

    // Create users in non-alphabetical order
    repo.upsert(UpsertUserParam {
        discord_id: "333333333".to_string(),
        name: "Zoe".to_string(),
        is_admin: None,
    })
    .await?;

    repo.upsert(UpsertUserParam {
        discord_id: "111111111".to_string(),
        name: "Alice".to_string(),
        is_admin: None,
    })
    .await?;

    repo.upsert(UpsertUserParam {
        discord_id: "222222222".to_string(),
        name: "Bob".to_string(),
        is_admin: None,
    })
    .await?;

    let result = repo.get_all_paginated(0, 10).await;

    assert!(result.is_ok());
    let (users, _) = result.unwrap();
    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[1].name, "Bob");
    assert_eq!(users[2].name, "Zoe");

    Ok(())
}
