use super::*;

/// Tests retrieving a fleet by ID without field values.
///
/// Verifies that the repository successfully retrieves a fleet record
/// by its ID when it has no associated field values.
///
/// Expected: Ok(Some((fleet, empty_map)))
#[tokio::test]
async fn returns_fleet_without_field_values() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet with all dependencies
    let (_user, _guild, _ping_format, _category, fleet) =
        factory::helpers::create_fleet_with_dependencies(db).await?;

    let repo = FleetRepository::new(db);
    let result = repo.get_by_id(fleet.id).await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.is_some());
    let (retrieved_fleet, field_values) = data.unwrap();
    assert_eq!(retrieved_fleet.id, fleet.id);
    assert_eq!(retrieved_fleet.name, fleet.name);
    assert!(field_values.is_empty());

    Ok(())
}

/// Tests retrieving a fleet by ID with field values.
///
/// Verifies that the repository successfully retrieves a fleet and all
/// its associated custom field values.
///
/// Expected: Ok(Some((fleet, field_values_map)))
#[tokio::test]
async fn returns_fleet_with_field_values() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, _guild, ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    // Create ping format fields
    let field1 =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Field 1", 1)
            .await?;
    let field2 =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Field 2", 2)
            .await?;

    let fleet_time = Utc::now() + Duration::hours(2);
    let mut field_values = HashMap::new();
    field_values.insert(field1.id, "Value 1".to_string());
    field_values.insert(field2.id, "Value 2".to_string());

    let repo = FleetRepository::new(db);
    let fleet = repo
        .create(CreateFleetParams {
            category_id: category.id,
            name: "Test Fleet".to_string(),
            commander_id: user.discord_id.parse().unwrap(),
            fleet_time,
            description: None,
            hidden: false,
            disable_reminder: false,
            field_values: field_values.clone(),
        })
        .await?;

    let result = repo.get_by_id(fleet.id).await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.is_some());
    let (retrieved_fleet, retrieved_field_values) = data.unwrap();
    assert_eq!(retrieved_fleet.id, fleet.id);
    assert_eq!(retrieved_field_values, field_values);

    Ok(())
}

/// Tests retrieving non-existent fleet.
///
/// Verifies that the repository returns None when querying for a fleet
/// ID that doesn't exist in the database.
///
/// Expected: Ok(None)
#[tokio::test]
async fn returns_none_for_nonexistent_fleet() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = FleetRepository::new(db);
    let result = repo.get_by_id(999999).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    Ok(())
}
