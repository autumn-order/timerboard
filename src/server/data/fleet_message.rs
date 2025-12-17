use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};

pub struct FleetMessageRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FleetMessageRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new fleet message record
    ///
    /// # Arguments
    /// - `fleet_id`: ID of the fleet
    /// - `channel_id`: Discord channel ID where message was posted
    /// - `message_id`: Discord message ID
    /// - `message_type`: Type of message (creation, reminder, formup)
    ///
    /// # Returns
    /// - `Ok(Model)`: The created fleet message record
    /// - `Err(DbErr)`: Database error
    pub async fn create(
        &self,
        fleet_id: i32,
        channel_id: u64,
        message_id: u64,
        message_type: &str,
    ) -> Result<entity::fleet_message::Model, DbErr> {
        entity::fleet_message::ActiveModel {
            fleet_id: ActiveValue::Set(fleet_id),
            channel_id: ActiveValue::Set(channel_id.to_string()),
            message_id: ActiveValue::Set(message_id.to_string()),
            message_type: ActiveValue::Set(message_type.to_string()),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    /// Gets all messages for a fleet
    ///
    /// # Returns
    /// - `Ok(Vec<Model>)`: Vector of fleet messages
    /// - `Err(DbErr)`: Database error
    pub async fn get_by_fleet_id(
        &self,
        fleet_id: i32,
    ) -> Result<Vec<entity::fleet_message::Model>, DbErr> {
        entity::prelude::FleetMessage::find()
            .filter(entity::fleet_message::Column::FleetId.eq(fleet_id))
            .all(self.db)
            .await
    }
}
