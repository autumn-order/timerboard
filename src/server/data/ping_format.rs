use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};

pub struct PingFormatRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PingFormatRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new ping format
    pub async fn create(
        &self,
        guild_id: u64,
        name: String,
    ) -> Result<entity::ping_format::Model, DbErr> {
        entity::ping_format::ActiveModel {
            guild_id: ActiveValue::Set(guild_id.to_string()),
            name: ActiveValue::Set(name),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    /// Gets paginated ping formats for a guild
    pub async fn get_by_guild_id_paginated(
        &self,
        guild_id: u64,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<entity::ping_format::Model>, u64), DbErr> {
        let paginator = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::GuildId.eq(guild_id.to_string()))
            .order_by_asc(entity::ping_format::Column::Name)
            .paginate(self.db, per_page);

        let total = paginator.num_items().await?;
        let ping_formats = paginator.fetch_page(page).await?;

        Ok((ping_formats, total))
    }

    /// Updates a ping format's name
    pub async fn update(&self, id: i32, name: String) -> Result<entity::ping_format::Model, DbErr> {
        let ping_format = entity::prelude::PingFormat::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Ping format with id {} not found",
                id
            )))?;

        let mut active_model: entity::ping_format::ActiveModel = ping_format.into();
        active_model.name = ActiveValue::Set(name);

        active_model.update(self.db).await
    }

    /// Deletes a ping format
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::PingFormat::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// Checks if a ping format exists and belongs to the specified guild
    pub async fn exists_in_guild(&self, id: i32, guild_id: u64) -> Result<bool, DbErr> {
        let count = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::Id.eq(id))
            .filter(entity::ping_format::Column::GuildId.eq(guild_id.to_string()))
            .count(self.db)
            .await?;

        Ok(count > 0)
    }

    /// Gets the count of fleet categories using a specific ping format
    pub async fn get_fleet_category_count(&self, ping_format_id: i32) -> Result<u64, DbErr> {
        entity::prelude::FleetCategory::find()
            .filter(entity::fleet_category::Column::PingFormatId.eq(ping_format_id))
            .count(self.db)
            .await
    }
}

pub struct PingFormatFieldRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PingFormatFieldRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new ping format field
    pub async fn create(
        &self,
        ping_format_id: i32,
        name: String,
        priority: i32,
    ) -> Result<entity::ping_format_field::Model, DbErr> {
        entity::ping_format_field::ActiveModel {
            ping_format_id: ActiveValue::Set(ping_format_id),
            name: ActiveValue::Set(name),
            priority: ActiveValue::Set(priority),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    /// Gets all fields for a ping format, ordered by priority
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i32,
    ) -> Result<Vec<entity::ping_format_field::Model>, DbErr> {
        entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format_id))
            .order_by_asc(entity::ping_format_field::Column::Priority)
            .all(self.db)
            .await
    }

    /// Updates a ping format field's name and priority
    pub async fn update(
        &self,
        id: i32,
        name: String,
        priority: i32,
    ) -> Result<entity::ping_format_field::Model, DbErr> {
        let field = entity::prelude::PingFormatField::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Ping format field with id {} not found",
                id
            )))?;

        let mut active_model: entity::ping_format_field::ActiveModel = field.into();
        active_model.name = ActiveValue::Set(name);
        active_model.priority = ActiveValue::Set(priority);

        active_model.update(self.db).await
    }

    /// Deletes a ping format field
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::PingFormatField::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }
}
