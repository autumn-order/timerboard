use migration::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use serenity::all::User as DiscordUser;

pub struct DiscordUserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordUserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, user: DiscordUser) -> Result<entity::discord_user::Model, DbErr> {
        entity::prelude::DiscordUser::insert(entity::discord_user::ActiveModel {
            discord_id: ActiveValue::Set(user.id.get() as i32),
            name: ActiveValue::Set(user.name),
            ..Default::default()
        })
        // Update user name in case it may have changed since last login
        .on_conflict(
            OnConflict::column(entity::discord_user::Column::DiscordId)
                .update_columns([entity::discord_user::Column::Name])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await
    }
}
