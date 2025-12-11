use migration::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serenity::all::{Guild, Role, RoleId};
use std::collections::HashMap;

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
}

pub struct DiscordGuildRoleRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildRoleRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(
        &self,
        guild_id: u64,
        role: &Role,
    ) -> Result<entity::discord_guild_role::Model, DbErr> {
        entity::prelude::DiscordGuildRole::insert(entity::discord_guild_role::ActiveModel {
            guild_id: ActiveValue::Set(guild_id as i64),
            role_id: ActiveValue::Set(role.id.get() as i64),
            name: ActiveValue::Set(role.name.clone()),
            color: ActiveValue::Set(format!("#{:06X}", role.colour.0)),
            position: ActiveValue::Set(role.position as i16),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::column(entity::discord_guild_role::Column::RoleId)
                .update_columns([
                    entity::discord_guild_role::Column::Name,
                    entity::discord_guild_role::Column::Color,
                    entity::discord_guild_role::Column::Position,
                ])
                .to_owned(),
        )
        .exec_with_returning(self.db)
        .await
    }

    pub async fn upsert_many(
        &self,
        guild_id: u64,
        roles: &HashMap<RoleId, Role>,
    ) -> Result<Vec<entity::discord_guild_role::Model>, DbErr> {
        let mut results = Vec::new();

        for (_, role) in roles {
            let model = self.upsert(guild_id, role).await?;
            results.push(model);
        }

        Ok(results)
    }

    pub async fn delete(&self, role_id: u64) -> Result<(), DbErr> {
        entity::prelude::DiscordGuildRole::delete_many()
            .filter(entity::discord_guild_role::Column::RoleId.eq(role_id))
            .exec(self.db)
            .await?;
        Ok(())
    }

    pub async fn get_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Vec<entity::discord_guild_role::Model>, DbErr> {
        entity::prelude::DiscordGuildRole::find()
            .filter(entity::discord_guild_role::Column::GuildId.eq(guild_id))
            .all(self.db)
            .await
    }
}
