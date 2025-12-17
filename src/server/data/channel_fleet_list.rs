use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::server::error::AppError;

pub struct ChannelFleetListRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ChannelFleetListRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Gets the fleet list message for a channel
    pub async fn get_by_channel_id(
        &self,
        channel_id: &str,
    ) -> Result<Option<entity::channel_fleet_list::Model>, AppError> {
        let list = entity::prelude::ChannelFleetList::find()
            .filter(entity::channel_fleet_list::Column::ChannelId.eq(channel_id))
            .one(self.db)
            .await?;

        Ok(list)
    }

    /// Creates or updates the fleet list message for a channel
    pub async fn upsert(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<entity::channel_fleet_list::Model, AppError> {
        // Check if record exists
        let existing = self.get_by_channel_id(channel_id).await?;

        let now = chrono::Utc::now();

        if let Some(existing) = existing {
            // Update existing record
            let mut active: entity::channel_fleet_list::ActiveModel = existing.into();
            active.message_id = ActiveValue::Set(message_id.to_string());
            active.last_message_at = ActiveValue::Set(now);
            active.updated_at = ActiveValue::Set(now);
            Ok(active.update(self.db).await?)
        } else {
            // Create new record
            let new_record = entity::channel_fleet_list::ActiveModel {
                id: ActiveValue::NotSet,
                channel_id: ActiveValue::Set(channel_id.to_string()),
                message_id: ActiveValue::Set(message_id.to_string()),
                last_message_at: ActiveValue::Set(now),
                created_at: ActiveValue::Set(now),
                updated_at: ActiveValue::Set(now),
            };
            Ok(new_record.insert(self.db).await?)
        }
    }
}
