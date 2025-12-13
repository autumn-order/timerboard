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
        guild_id: i64,
        name: String,
    ) -> Result<entity::ping_format::Model, DbErr> {
        entity::ping_format::ActiveModel {
            guild_id: ActiveValue::Set(guild_id),
            name: ActiveValue::Set(name),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    /// Gets a ping format by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<entity::ping_format::Model>, DbErr> {
        entity::prelude::PingFormat::find_by_id(id)
            .one(self.db)
            .await
    }

    /// Gets paginated ping formats for a guild
    pub async fn get_by_guild_id_paginated(
        &self,
        guild_id: i64,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<entity::ping_format::Model>, u64), DbErr> {
        let paginator = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::GuildId.eq(guild_id))
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
    pub async fn exists_in_guild(&self, id: i32, guild_id: i64) -> Result<bool, DbErr> {
        let count = entity::prelude::PingFormat::find()
            .filter(entity::ping_format::Column::Id.eq(id))
            .filter(entity::ping_format::Column::GuildId.eq(guild_id))
            .count(self.db)
            .await?;

        Ok(count > 0)
    }
}
