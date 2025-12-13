use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter,
};

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
        ping_format_id: i64,
        name: String,
    ) -> Result<entity::ping_format_field::Model, DbErr> {
        entity::ping_format_field::ActiveModel {
            ping_format_id: ActiveValue::Set(ping_format_id),
            name: ActiveValue::Set(name),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    /// Gets all fields for a ping format
    pub async fn get_by_ping_format_id(
        &self,
        ping_format_id: i64,
    ) -> Result<Vec<entity::ping_format_field::Model>, DbErr> {
        entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format_id))
            .all(self.db)
            .await
    }

    /// Updates a ping format field's name
    pub async fn update(
        &self,
        id: i32,
        name: String,
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

        active_model.update(self.db).await
    }

    /// Deletes a ping format field
    pub async fn delete(&self, id: i32) -> Result<(), DbErr> {
        entity::prelude::PingFormatField::delete_by_id(id)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// Deletes all fields for a ping format
    pub async fn delete_by_ping_format_id(&self, ping_format_id: i64) -> Result<(), DbErr> {
        entity::prelude::PingFormatField::delete_many()
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format_id))
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// Checks if a field belongs to a specific ping format
    pub async fn exists_in_ping_format(&self, id: i32, ping_format_id: i64) -> Result<bool, DbErr> {
        let count = entity::prelude::PingFormatField::find()
            .filter(entity::ping_format_field::Column::Id.eq(id))
            .filter(entity::ping_format_field::Column::PingFormatId.eq(ping_format_id))
            .count(self.db)
            .await?;

        Ok(count > 0)
    }
}
