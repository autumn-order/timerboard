use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use std::collections::HashMap;

pub struct FleetRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet with field values
    ///
    /// # Arguments
    /// - `category_id`: ID of the fleet category
    /// - `name`: Fleet name
    /// - `commander_id`: Discord ID of the fleet commander (u64, stored as string)
    /// - `fleet_time`: DateTime when the fleet will occur
    /// - `description`: Optional description of the fleet
    /// - `field_values`: HashMap of field_id -> value for ping format fields
    ///
    /// # Returns
    /// - `Ok(Model)`: The created fleet
    /// - `Err(DbErr)`: Database error
    pub async fn create(
        &self,
        category_id: i32,
        name: String,
        commander_id: u64,
        fleet_time: DateTime<Utc>,
        description: Option<String>,
        field_values: HashMap<i32, String>,
        hidden: bool,
        disable_reminder: bool,
    ) -> Result<entity::fleet::Model, DbErr> {
        // Create the fleet
        let fleet = entity::fleet::ActiveModel {
            category_id: ActiveValue::Set(category_id),
            name: ActiveValue::Set(name),
            commander_id: ActiveValue::Set(commander_id.to_string()),
            fleet_time: ActiveValue::Set(fleet_time),
            description: ActiveValue::Set(description),
            hidden: ActiveValue::Set(hidden),
            disable_reminder: ActiveValue::Set(disable_reminder),
            created_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        // Insert field values
        for (field_id, value) in field_values {
            entity::fleet_field_value::ActiveModel {
                fleet_id: ActiveValue::Set(fleet.id),
                field_id: ActiveValue::Set(field_id),
                value: ActiveValue::Set(value),
            }
            .insert(self.db)
            .await?;
        }

        Ok(fleet)
    }

    /// Gets a fleet by ID with its field values
    ///
    /// # Returns
    /// - `Ok(Some((fleet, field_values)))`: Fleet and map of field_id -> value
    /// - `Ok(None)`: Fleet not found
    /// - `Err(DbErr)`: Database error
    pub async fn get_by_id(
        &self,
        id: i32,
    ) -> Result<Option<(entity::fleet::Model, HashMap<i32, String>)>, DbErr> {
        let fleet = entity::prelude::Fleet::find_by_id(id).one(self.db).await?;

        if let Some(fleet) = fleet {
            let field_values = entity::prelude::FleetFieldValue::find()
                .filter(entity::fleet_field_value::Column::FleetId.eq(id))
                .all(self.db)
                .await?
                .into_iter()
                .map(|fv| (fv.field_id, fv.value))
                .collect();

            Ok(Some((fleet, field_values)))
        } else {
            Ok(None)
        }
    }

    /// Gets paginated fleets for a guild, ordered by fleet_time (upcoming first)
    ///
    /// Filters fleets to only include:
    /// - Fleets in categories the user can view (or all if category_ids is None for admins)
    /// - Fleets that are not older than 1 hour from the current time
    ///
    /// # Arguments
    /// - `guild_id`: Discord guild ID (u64)
    /// - `page`: Page number (0-indexed)
    /// - `per_page`: Number of items per page
    /// - `viewable_category_ids`: Optional list of category IDs the user can view (None means all categories - admin bypass)
    ///
    /// # Returns
    /// - `Ok((fleets, total))`: Vector of fleets and total count
    /// - `Err(DbErr)`: Database error
    pub async fn get_paginated_by_guild(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
        viewable_category_ids: Option<Vec<i32>>,
    ) -> Result<(Vec<entity::fleet::Model>, u64), DbErr> {
        use entity::fleet_category;
        use sea_orm::JoinType;

        let guild_id_str = guild_id.to_string();

        // Calculate cutoff time (1 hour ago)
        let cutoff_time = Utc::now() - chrono::Duration::hours(1);

        let mut query = entity::prelude::Fleet::find()
            .join(
                JoinType::InnerJoin,
                entity::fleet::Relation::FleetCategory.def(),
            )
            .filter(fleet_category::Column::GuildId.eq(guild_id_str.as_str()))
            .filter(entity::fleet::Column::FleetTime.gte(cutoff_time))
            .order_by_asc(entity::fleet::Column::FleetTime);

        // If viewable_category_ids is provided, filter by those categories
        if let Some(category_ids) = viewable_category_ids {
            if category_ids.is_empty() {
                // User has no viewable categories, return empty result
                return Ok((Vec::new(), 0));
            }
            query = query.filter(entity::fleet::Column::CategoryId.is_in(category_ids));
        }

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let fleets = paginator.fetch_page(page).await?;

        Ok((fleets, total))
    }

    /// Gets paginated fleets for a specific category
    ///
    /// # Arguments
    /// - `category_id`: Fleet category ID
    /// - `page`: Page number (0-indexed)
    /// - `per_page`: Number of items per page
    ///

    /// Deletes a fleet by ID
    ///
    /// # Arguments
    /// - `id`: Fleet ID
    ///
    /// # Returns
    /// - `Ok(())`: Fleet deleted successfully
    /// - `Err(DbErr)`: Database error
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::Fleet::delete_by_id(id)
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Updates a fleet
    ///
    /// # Arguments
    /// - `id`: Fleet ID
    /// - `name`: Optional new fleet name
    /// - `fleet_time`: Optional new fleet time
    /// - `description`: Optional new description (None removes description)
    /// - `field_values`: Optional new field values (replaces all existing values)
    ///
    /// # Returns
    /// - `Ok(Model)`: The updated fleet
    /// - `Err(DbErr)`: Database error
    pub async fn update(
        &self,
        id: i32,
        category_id: Option<i32>,
        name: Option<String>,
        fleet_time: Option<DateTime<Utc>>,
        description: Option<Option<String>>,
        field_values: Option<HashMap<i32, String>>,
        hidden: Option<bool>,
        disable_reminder: Option<bool>,
    ) -> Result<entity::fleet::Model, DbErr> {
        let fleet = entity::prelude::Fleet::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Fleet {} not found", id)))?;

        let mut active_model: entity::fleet::ActiveModel = fleet.into();

        if let Some(category_id) = category_id {
            active_model.category_id = ActiveValue::Set(category_id);
        }
        if let Some(name) = name {
            active_model.name = ActiveValue::Set(name);
        }
        if let Some(fleet_time) = fleet_time {
            active_model.fleet_time = ActiveValue::Set(fleet_time);
        }
        if let Some(description) = description {
            active_model.description = ActiveValue::Set(description);
        }
        if let Some(hidden) = hidden {
            active_model.hidden = ActiveValue::Set(hidden);
        }
        if let Some(disable_reminder) = disable_reminder {
            active_model.disable_reminder = ActiveValue::Set(disable_reminder);
        }

        let updated_fleet = active_model.update(self.db).await?;

        // Update field values if provided
        if let Some(field_values) = field_values {
            // Delete existing field values
            entity::prelude::FleetFieldValue::delete_many()
                .filter(entity::fleet_field_value::Column::FleetId.eq(id))
                .exec(self.db)
                .await?;

            // Insert new field values
            for (field_id, value) in field_values {
                entity::fleet_field_value::ActiveModel {
                    fleet_id: ActiveValue::Set(id),
                    field_id: ActiveValue::Set(field_id),
                    value: ActiveValue::Set(value),
                }
                .insert(self.db)
                .await?;
            }
        }

        Ok(updated_fleet)
    }
}
