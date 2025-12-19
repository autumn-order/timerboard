use super::*;

/// Tests updating fleet name.
///
/// Verifies that the repository successfully updates a fleet's name
/// while leaving other fields unchanged.
///
/// Expected: Ok with name updated
#[tokio::test]
async fn updates_fleet_name() -> Result<(), DbErr> {
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
    let result = repo
        .update(UpdateFleetParams {
            id: fleet.id,
            category_id: None,
            name: Some("Updated Fleet Name".to_string()),
            fleet_time: None,
            description: None,
            hidden: None,
            disable_reminder: None,
            field_values: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "Updated Fleet Name");
    assert_eq!(updated.category_id, fleet.category_id);

    Ok(())
}

/// Tests updating fleet time.
///
/// Verifies that the repository successfully updates a fleet's
/// scheduled time.
///
/// Expected: Ok with fleet_time updated
#[tokio::test]
async fn updates_fleet_time() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet with all dependencies
    let (_user, _guild, _ping_format, _category, fleet) =
        factory::helpers::create_fleet_with_dependencies(db).await?;

    let new_time = Utc::now() + Duration::hours(5);
    let repo = FleetRepository::new(db);
    let result = repo
        .update(UpdateFleetParams {
            id: fleet.id,
            category_id: None,
            name: None,
            fleet_time: Some(new_time),
            description: None,
            hidden: None,
            disable_reminder: None,
            field_values: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.fleet_time, new_time);

    Ok(())
}

/// Tests updating multiple fields.
///
/// Verifies that the repository successfully updates multiple fleet
/// fields in a single operation.
///
/// Expected: Ok with all specified fields updated
#[tokio::test]
async fn updates_multiple_fields() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet with all dependencies
    let (_user, _guild, _ping_format, _category, fleet) =
        factory::helpers::create_fleet_with_dependencies(db).await?;

    let new_time = Utc::now() + Duration::hours(5);
    let repo = FleetRepository::new(db);
    let result = repo
        .update(UpdateFleetParams {
            id: fleet.id,
            category_id: None,
            name: Some("New Name".to_string()),
            fleet_time: Some(new_time),
            description: Some(Some("New Description".to_string())),
            hidden: Some(true),
            disable_reminder: Some(true),
            field_values: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.fleet_time, new_time);
    assert_eq!(updated.description, Some("New Description".to_string()));
    assert!(updated.hidden);
    assert!(updated.disable_reminder);

    Ok(())
}

/// Tests updating field values replaces existing values.
///
/// Verifies that providing new field values deletes all old values
/// and inserts the new ones.
///
/// Expected: Ok with field values replaced
#[tokio::test]
async fn replaces_field_values() -> Result<(), DbErr> {
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
    field_values.insert(field1.id, "Original Value 1".to_string());

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
            field_values,
        })
        .await?;

    // Update with new field values
    let mut new_field_values = HashMap::new();
    new_field_values.insert(field2.id, "New Value 2".to_string());

    repo.update(UpdateFleetParams {
        id: fleet.id,
        category_id: None,
        name: None,
        fleet_time: None,
        description: None,
        hidden: None,
        disable_reminder: None,
        field_values: Some(new_field_values.clone()),
    })
    .await?;

    // Verify field values were replaced
    let stored_values = entity::prelude::FleetFieldValue::find()
        .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
        .all(db)
        .await?;

    assert_eq!(stored_values.len(), 1);
    assert_eq!(stored_values[0].field_id, field2.id);
    assert_eq!(stored_values[0].value, "New Value 2");

    Ok(())
}

/// Tests updating with empty field values clears all values.
///
/// Verifies that providing an empty field values map deletes all
/// existing field values.
///
/// Expected: Ok with all field values deleted
#[tokio::test]
async fn clears_field_values_with_empty_map() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, _guild, ping_format, category) =
        factory::helpers::create_fleet_dependencies(db).await?;

    // Create ping format field
    let field =
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Field 1", 1)
            .await?;

    let fleet_time = Utc::now() + Duration::hours(2);
    let mut field_values = HashMap::new();
    field_values.insert(field.id, "Original Value".to_string());

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
            field_values,
        })
        .await?;

    // Update with empty field values
    repo.update(UpdateFleetParams {
        id: fleet.id,
        category_id: None,
        name: None,
        fleet_time: None,
        description: None,
        hidden: None,
        disable_reminder: None,
        field_values: Some(HashMap::new()),
    })
    .await?;

    // Verify field values were deleted
    let stored_values = entity::prelude::FleetFieldValue::find()
        .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
        .all(db)
        .await?;

    assert!(stored_values.is_empty());

    Ok(())
}

/// Tests updating category_id.
///
/// Verifies that the repository successfully updates a fleet's category.
///
/// Expected: Ok with category_id updated
#[tokio::test]
async fn updates_category_id() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet dependencies
    let (user, guild, ping_format, category1) =
        factory::helpers::create_fleet_dependencies(db).await?;

    // Create second category
    let category2 =
        factory::fleet_category::create_category(db, &guild.guild_id, ping_format.id).await?;

    let fleet_time = Utc::now() + Duration::hours(2);
    let repo = FleetRepository::new(db);
    let fleet = repo
        .create(CreateFleetParams {
            category_id: category1.id,
            name: "Test Fleet".to_string(),
            commander_id: user.discord_id.parse().unwrap(),
            fleet_time,
            description: None,
            hidden: false,
            disable_reminder: false,
            field_values: HashMap::new(),
        })
        .await?;

    // Update category
    let result = repo
        .update(UpdateFleetParams {
            id: fleet.id,
            category_id: Some(category2.id),
            name: None,
            fleet_time: None,
            description: None,
            hidden: None,
            disable_reminder: None,
            field_values: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.category_id, category2.id);

    Ok(())
}

/// Tests updating non-existent fleet fails.
///
/// Verifies that attempting to update a fleet that doesn't exist
/// returns a RecordNotFound error.
///
/// Expected: Err(DbErr::RecordNotFound)
#[tokio::test]
async fn fails_for_nonexistent_fleet() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = FleetRepository::new(db);
    let result = repo
        .update(UpdateFleetParams {
            id: 999999,
            category_id: None,
            name: Some("New Name".to_string()),
            fleet_time: None,
            description: None,
            hidden: None,
            disable_reminder: None,
            field_values: None,
        })
        .await;

    assert!(result.is_err());
    match result {
        Err(DbErr::RecordNotFound(_)) => (),
        _ => panic!("Expected RecordNotFound error"),
    }

    Ok(())
}

/// Tests updating with None values preserves existing values.
///
/// Verifies that when update parameters are None, the original
/// values are preserved.
///
/// Expected: Ok with original values unchanged
#[tokio::test]
async fn preserves_unspecified_fields() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create fleet with all dependencies
    let (_user, _guild, _ping_format, _category, fleet) =
        factory::helpers::create_fleet_with_dependencies(db).await?;

    let original_name = fleet.name.clone();
    let original_time = fleet.fleet_time;

    let repo = FleetRepository::new(db);
    let result = repo
        .update(UpdateFleetParams {
            id: fleet.id,
            category_id: None,
            name: None,
            fleet_time: None,
            description: Some(Some("New Description".to_string())),
            hidden: None,
            disable_reminder: None,
            field_values: None,
        })
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, original_name);
    assert_eq!(updated.fleet_time, original_time);
    assert_eq!(updated.description, Some("New Description".to_string()));

    Ok(())
}
