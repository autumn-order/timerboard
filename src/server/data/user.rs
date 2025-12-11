use migration::OnConflict;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
};
use serenity::all::User as DiscordUser;

pub struct UserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(
        &self,
        user: DiscordUser,
        is_admin: bool,
    ) -> Result<entity::user::Model, DbErr> {
        entity::prelude::User::insert(entity::user::ActiveModel {
            discord_id: ActiveValue::Set(user.id.get() as i64),
            name: ActiveValue::Set(user.name),
            admin: ActiveValue::Set(is_admin),
            ..Default::default()
        })
        // Update user name in case it may have changed since last login
        .on_conflict(
            OnConflict::column(entity::user::Column::DiscordId)
                .update_columns([entity::user::Column::Name])
                .update_columns([entity::user::Column::Admin])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await
    }

    pub async fn find_by_id(&self, user_id: i32) -> Result<Option<entity::user::Model>, DbErr> {
        entity::prelude::User::find_by_id(user_id)
            .one(self.db)
            .await
    }

    /// Checks if any admin users exist in the database.
    ///
    /// # Returns
    /// - `Ok(true)` if at least one admin user exists
    /// - `Ok(false)` if no admin users exist
    /// - `Err(DbErr)` if the database query fails
    pub async fn admin_exists(&self) -> Result<bool, DbErr> {
        let admin_count = entity::prelude::User::find()
            .filter(entity::user::Column::Admin.eq(true))
            .count(self.db)
            .await?;

        Ok(admin_count > 0)
    }
}
