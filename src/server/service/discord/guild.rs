use sea_orm::DatabaseConnection;

use crate::{
    model::discord::DiscordGuildDto,
    server::{data::discord::DiscordGuildRepository, error::AppError},
};

pub struct DiscordGuildService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> DiscordGuildService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_all(&self) -> Result<Vec<DiscordGuildDto>, AppError> {
        let guild_repo = DiscordGuildRepository::new(self.db);

        let guilds = guild_repo
            .get_all()
            .await?
            .into_iter()
            .map(|g| DiscordGuildDto {
                id: g.id,
                guild_id: g.guild_id,
                name: g.name,
                icon_hash: g.icon_hash,
            })
            .collect();

        Ok(guilds)
    }

    pub async fn get_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Option<DiscordGuildDto>, AppError> {
        let guild_repo = DiscordGuildRepository::new(self.db);

        let guild = guild_repo.find_by_guild_id(guild_id).await?;

        Ok(guild.map(|g| DiscordGuildDto {
            id: g.id,
            guild_id: g.guild_id,
            name: g.name,
            icon_hash: g.icon_hash,
        }))
    }
}
