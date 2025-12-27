//! Fleet data repository for database operations.
//!
//! This module provides the `FleetRepository` for managing fleet records in the database.
//! Fleets represent scheduled operations with commanders, categories, custom fields, and
//! notification settings. The repository handles creation, updates, queries, and deletion
//! with proper conversion between entity models and parameter models at the infrastructure
//! boundary.

use chrono::Utc;
use dioxus_logger::tracing;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use std::collections::HashMap;

use crate::server::{
    error::AppError,
    model::fleet::{CreateFleetParam, Fleet, UpdateFleetParam},
};

/// Repository providing database operations for fleet management.
///
/// This struct holds a reference to the database connection and provides methods
/// for creating, reading, updating, and deleting fleet records.
pub struct FleetRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetRepository<'a> {
    /// Creates a new FleetRepository instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    ///
    /// # Returns
    /// - `FleetRepository` - New repository instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet with field values.
    ///
    /// Inserts a new fleet record into the database and creates associated field value
    /// records for custom ping format fields. The fleet is created with a timestamp and
    /// all field values are inserted within the same operation sequence.
    ///
    /// # Arguments
    /// - `params` - Create parameters containing all fleet creation data including field values
    ///
    /// # Returns
    /// - `Ok(Fleet)` - The created fleet with generated ID
    /// - `Err(AppError::Database)` - Database error during insert operation (including foreign key violations)
    /// - `Err(AppError::InternalError(ParseStringId))` - Failed to parse ID from String
    pub async fn create(&self, param: CreateFleetParam) -> Result<Fleet, AppError> {
        // Create the fleet
        let entity = entity::fleet::ActiveModel {
            category_id: ActiveValue::Set(param.category_id),
            name: ActiveValue::Set(param.name),
            commander_id: ActiveValue::Set(param.commander_id.to_string()),
            fleet_time: ActiveValue::Set(param.fleet_time),
            description: ActiveValue::Set(param.description),
            hidden: ActiveValue::Set(param.hidden),
            disable_reminder: ActiveValue::Set(param.disable_reminder),
            created_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        // Insert field values
        for (field_id, value) in param.field_values {
            entity::fleet_field_value::ActiveModel {
                fleet_id: ActiveValue::Set(entity.id),
                field_id: ActiveValue::Set(field_id),
                value: ActiveValue::Set(value),
            }
            .insert(self.db)
            .await?;
        }

        Ok(Fleet::from_entity(entity)?)
    }

    /// Gets a fleet by ID with its field values.
    ///
    /// Retrieves a fleet and all associated custom field values. Returns both the fleet
    /// data and a map of field_id to field value for easy lookup. Used when displaying
    /// or editing fleet details.
    ///
    /// # Arguments
    /// - `id` - ID of the fleet to retrieve
    ///
    /// # Returns
    /// - `Ok(Some((fleet, field_values)))` - Fleet param and map of field_id to value
    /// - `Ok(None)` - No fleet found with that ID
    /// - `Err(AppError::Database)` - Database error during query
    /// - `Err(AppError::InternalError(ParseStringId))` - Failed to parse ID from String
    pub async fn get_by_id(
        &self,
        id: i32,
    ) -> Result<Option<(Fleet, HashMap<i32, String>)>, AppError> {
        let entity = entity::prelude::Fleet::find_by_id(id).one(self.db).await?;

        if let Some(entity) = entity {
            let field_values = entity::prelude::FleetFieldValue::find()
                .filter(entity::fleet_field_value::Column::FleetId.eq(id))
                .all(self.db)
                .await?
                .into_iter()
                .map(|fv| (fv.field_id, fv.value))
                .collect();

            Ok(Some((Fleet::from_entity(entity)?, field_values)))
        } else {
            Ok(None)
        }
    }

    /// Gets upcoming non-hidden fleets for specific categories.
    ///
    /// Retrieves all fleets for the specified category IDs that:
    /// - Have a fleet_time greater than the provided time
    /// - Are not hidden
    /// - Are ordered by fleet_time in ascending order
    ///
    /// This is used for building the upcoming fleets list in Discord channels.
    ///
    /// # Arguments
    /// - `category_ids` - List of category IDs to filter by
    /// - `after_time` - Only include fleets with fleet_time after this time
    ///
    /// # Returns
    /// - `Ok(Vec<Fleet>)` - Vector of upcoming fleets ordered by time
    /// - `Err(AppError::Database)` - Database error during query
    /// - `Err(AppError::InternalError(ParseStringId))` - Failed to parse ID from String
    pub async fn get_upcoming_by_categories(
        &self,
        category_ids: Vec<i32>,
        after_time: chrono::DateTime<Utc>,
    ) -> Result<Vec<Fleet>, AppError> {
        let entities = entity::prelude::Fleet::find()
            .filter(entity::fleet::Column::CategoryId.is_in(category_ids))
            .filter(entity::fleet::Column::FleetTime.gt(after_time))
            .filter(entity::fleet::Column::Hidden.eq(false))
            .order_by_asc(entity::fleet::Column::FleetTime)
            .all(self.db)
            .await?;

        entities
            .into_iter()
            .map(Fleet::from_entity)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Gets paginated fleets for a guild, ordered by fleet_time (upcoming first).
    ///
    /// Filters fleets to only include:
    /// - Fleets in categories the user can view (or all if category_ids is None for admins)
    /// - Fleets that are not older than 1 hour from the current time
    ///
    /// The cutoff time prevents showing very old completed fleets while allowing recently
    /// started fleets to remain visible briefly. Results are ordered by fleet_time in
    /// ascending order so upcoming fleets appear first.
    ///
    /// # Arguments
    /// - `guild_id` - Discord guild ID as u64
    /// - `page` - Zero-indexed page number
    /// - `per_page` - Number of fleets to return per page
    /// - `viewable_category_ids` - Optional list of category IDs the user can view (None means all categories - admin bypass)
    ///
    /// # Returns
    /// - `Ok((fleets, total))` - Vector of fleets for the page and total count
    /// - `Err(DbErr)` - Database error during pagination query
    pub async fn get_paginated_by_guild(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
        viewable_category_ids: Option<Vec<i32>>,
    ) -> Result<(Vec<Fleet>, u64), DbErr> {
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
        let entities = paginator.fetch_page(page).await?;
        let fleets = entities
            .into_iter()
            .filter_map(|entity| match Fleet::from_entity(entity) {
                Ok(fleet) => Some(fleet),
                Err(e) => {
                    tracing::error!("Failed to convert fleet entity to domain model: {}", e);
                    None
                }
            })
            .collect();

        Ok((fleets, total))
    }

    /// Deletes a fleet by ID.
    ///
    /// Deletes the fleet with the specified ID. Associated field values and fleet messages
    /// are automatically deleted due to CASCADE foreign key constraints.
    ///
    /// # Arguments
    /// - `id` - ID of the fleet to delete
    ///
    /// # Returns
    /// - `Ok(())` - Fleet deleted successfully (or didn't exist)
    /// - `Err(DbErr)` - Database error during delete operation
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::Fleet::delete_by_id(id)
            .exec(self.db)
            .await?;
        Ok(())
    }

    /// Updates a fleet.
    ///
    /// Updates an existing fleet's properties. Only provided fields in the params are updated,
    /// allowing partial updates. If field_values are provided, all existing field values are
    /// deleted and replaced with the new values.
    ///
    /// # Arguments
    /// - `params` - Update parameters containing fleet ID and optional new values
    ///
    /// # Returns
    /// - `Ok(Fleet)` - The updated fleet with new values
    /// - `Err(DbErr::RecordNotFound)` - No fleet exists with the specified ID
    /// - `Err(DbErr)` - Other database error during update operation
    /// - `Err(AppError::InternalError(ParseStringId))` - Failed to parse ID from String
    pub async fn update(&self, params: UpdateFleetParam) -> Result<Fleet, AppError> {
        let id = params.id;
        let fleet = entity::prelude::Fleet::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Fleet {} not found", id)))?;

        let mut active_model: entity::fleet::ActiveModel = fleet.into();

        if let Some(category_id) = params.category_id {
            active_model.category_id = ActiveValue::Set(category_id);
        }
        if let Some(name) = params.name {
            active_model.name = ActiveValue::Set(name);
        }
        if let Some(fleet_time) = params.fleet_time {
            active_model.fleet_time = ActiveValue::Set(fleet_time);
        }
        if let Some(description) = params.description {
            active_model.description = ActiveValue::Set(description);
        }
        if let Some(hidden) = params.hidden {
            active_model.hidden = ActiveValue::Set(hidden);
        }
        if let Some(disable_reminder) = params.disable_reminder {
            active_model.disable_reminder = ActiveValue::Set(disable_reminder);
        }

        let updated_fleet = active_model.update(self.db).await?;

        // Update field values if provided
        if let Some(new_field_values) = params.field_values {
            // Delete existing field values
            entity::prelude::FleetFieldValue::delete_many()
                .filter(entity::fleet_field_value::Column::FleetId.eq(id))
                .exec(self.db)
                .await?;

            // Insert new field values
            for (field_id, value) in new_field_values {
                entity::fleet_field_value::ActiveModel {
                    fleet_id: ActiveValue::Set(id),
                    field_id: ActiveValue::Set(field_id),
                    value: ActiveValue::Set(value),
                }
                .insert(self.db)
                .await?;
            }
        }

        Ok(Fleet::from_entity(updated_fleet)?)
    }
}
