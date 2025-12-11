use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::Guild;

pub struct DiscordGuildRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, guild: Guild) -> Result<entity::discord_guild::Model, DbErr> {
        entity::prelude::DiscordGuild::insert(entity::discord_guild::ActiveModel {
            guild_id: ActiveValue::Set(guild.id.get() as i64),
            name: ActiveValue::Set(guild.name),
            icon_hash: ActiveValue::Set(guild.icon_hash.map(|i| i.to_string())),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::column(entity::discord_guild::Column::GuildId)
                .update_columns([entity::discord_guild::Column::Name])
                .update_columns([entity::discord_guild::Column::IconHash])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await
    }

    pub async fn get_all(&self) -> Result<Vec<entity::discord_guild::Model>, DbErr> {
        entity::prelude::DiscordGuild::find().all(self.db).await
    }

    pub async fn find_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Option<entity::discord_guild::Model>, DbErr> {
        entity::prelude::DiscordGuild::find()
            .filter(entity::discord_guild::Column::GuildId.eq(guild_id as i64))
            .one(self.db)
            .await
    }
}
