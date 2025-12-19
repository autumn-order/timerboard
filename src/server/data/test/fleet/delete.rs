use super::*;

/// Tests deleting a fleet.
///
/// Verifies that the repository successfully deletes a fleet record
/// by its ID.
///
/// Expected: Ok with fleet deleted
#[tokio::test]
async fn deletes_fleet() -> Result<(), DbErr> {
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
    let result = repo.delete(fleet.id).await;

    assert!(result.is_ok());

    // Verify fleet is deleted
    let check = entity::prelude::Fleet::find_by_id(fleet.id).one(db).await?;
    assert!(check.is_none());

    Ok(())
}

/// Tests deleting fleet cascades to field values.
///
/// Verifies that deleting a fleet also deletes all associated
/// field values due to CASCADE constraint.
///
/// Expected: Ok with fleet and field values deleted
#[tokio::test]
async fn cascades_to_field_values() -> Result<(), DbErr> {
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
        factory::ping_format_field::create_ping_format_field(db, ping_format.id, "Test Field", 1)
            .await?;

    let fleet_time = Utc::now() + Duration::hours(2);
    let mut field_values = HashMap::new();
    field_values.insert(field.id, "Test Value".to_string());

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

    // Verify field value exists
    let field_value_count = entity::prelude::FleetFieldValue::find()
        .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
        .count(db)
        .await?;
    assert_eq!(field_value_count, 1);

    // Delete fleet
    repo.delete(fleet.id).await?;

    // Verify field values are also deleted
    let field_value_count = entity::prelude::FleetFieldValue::find()
        .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
        .count(db)
        .await?;
    assert_eq!(field_value_count, 0);

    Ok(())
}

/// Tests deleting non-existent fleet succeeds.
///
/// Verifies that attempting to delete a fleet that doesn't exist
/// completes successfully without error.
///
/// Expected: Ok
#[tokio::test]
async fn succeeds_for_nonexistent_fleet() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let repo = FleetRepository::new(db);
    let result = repo.delete(999999).await;

    assert!(result.is_ok());

    Ok(())
}
