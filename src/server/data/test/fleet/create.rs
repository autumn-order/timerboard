use super::*;

/// Tests creating a new fleet without field values.
///
/// Verifies that the repository successfully creates a new fleet record
/// with the specified category_id, name, commander_id, fleet_time, description,
/// hidden, and disable_reminder values but no custom field values.
///
/// Expected: Ok with fleet created
#[tokio::test]
async fn creates_fleet_without_field_values() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, _guild, _ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    let fleet_time = Utc::now() + Duration::hours(2);
    let repo = FleetRepository::new(db);
    let result = repo
        .create(CreateFleetParams {
            category_id: category.id,
            name: "Test Fleet".to_string(),
            commander_id: user.discord_id.parse().unwrap(),
            fleet_time,
            description: Some("Test description".to_string()),
            hidden: false,
            disable_reminder: false,
            field_values: HashMap::new(),
        })
        .await;

    assert!(result.is_ok());
    let fleet = result.unwrap();
    assert_eq!(fleet.category_id, category.id);
    assert_eq!(fleet.name, "Test Fleet");
    assert_eq!(fleet.commander_id, user.discord_id);
    assert_eq!(fleet.fleet_time, fleet_time);
    assert_eq!(fleet.description, Some("Test description".to_string()));
    assert!(!fleet.hidden);
    assert!(!fleet.disable_reminder);

    Ok(())
}

/// Tests creating a fleet with field values.
///
/// Verifies that the repository successfully creates a fleet and its
/// associated custom field values in the database.
///
/// Expected: Ok with fleet and field values created
#[tokio::test]
async fn creates_fleet_with_field_values() -> Result<(), DbErr> {
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
    let result = repo
        .create(CreateFleetParams {
            category_id: category.id,
            name: "Test Fleet".to_string(),
            commander_id: user.discord_id.parse().unwrap(),
            fleet_time,
            description: Some("Test description".to_string()),
            hidden: false,
            disable_reminder: false,
            field_values: field_values.clone(),
        })
        .await;

    assert!(result.is_ok());
    let fleet = result.unwrap();

    // Verify field values were created
    let stored_values = entity::prelude::FleetFieldValue::find()
        .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
        .all(db)
        .await?;

    assert_eq!(stored_values.len(), 2);
    let stored_map: HashMap<i32, String> = stored_values
        .into_iter()
        .map(|fv| (fv.field_id, fv.value))
        .collect();
    assert_eq!(stored_map, field_values);

    Ok(())
}

/// Tests foreign key constraint on category_id.
///
/// Verifies that the repository returns an error when attempting to create
/// a fleet with a category_id that doesn't exist in the database.
///
/// Expected: Err(DbErr) due to foreign key constraint violation
#[tokio::test]
async fn fails_for_nonexistent_category() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let fleet_time = Utc::now() + Duration::hours(2);
    let repo = FleetRepository::new(db);
    let result = repo
        .create(CreateFleetParams {
            category_id: 999999, // Non-existent category
            name: "Test Fleet".to_string(),
            commander_id: 123456789,
            fleet_time,
            description: None,
            hidden: false,
            disable_reminder: false,
            field_values: HashMap::new(),
        })
        .await;

    assert!(result.is_err());

    Ok(())
}

/// Tests creating fleet with optional fields as None.
///
/// Verifies that the repository successfully creates a fleet when
/// the description field is None.
///
/// Expected: Ok with fleet created with None description
#[tokio::test]
async fn creates_fleet_with_none_description() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, _guild, _ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    let fleet_time = Utc::now() + Duration::hours(2);
    let repo = FleetRepository::new(db);
    let result = repo
        .create(CreateFleetParams {
            category_id: category.id,
            name: "Test Fleet".to_string(),
            commander_id: user.discord_id.parse().unwrap(),
            fleet_time,
            description: None,
            hidden: false,
            disable_reminder: false,
            field_values: HashMap::new(),
        })
        .await;

    assert!(result.is_ok());
    let fleet = result.unwrap();
    assert!(fleet.description.is_none());

    Ok(())
}
