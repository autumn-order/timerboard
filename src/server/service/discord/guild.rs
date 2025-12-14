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

        let guilds: Result<Vec<_>, _> = guild_repo
            .get_all()
            .await?
            .into_iter()
            .map(|g| {
                let guild_id = g.guild_id.parse::<u64>().map_err(|e| {
                    AppError::InternalError(format!("Failed to parse guild_id: {}", e))
                })?;
                Ok(DiscordGuildDto {
                    id: g.id,
                    guild_id,
                    name: g.name,
                    icon_hash: g.icon_hash,
                })
            })
            .collect();

        guilds
    }

    pub async fn get_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Option<DiscordGuildDto>, AppError> {
        let guild_repo = DiscordGuildRepository::new(self.db);

        let guild = guild_repo.find_by_guild_id(guild_id).await?;

        guild
            .map(|g| {
                let guild_id = g.guild_id.parse::<u64>().map_err(|e| {
                    AppError::InternalError(format!("Failed to parse guild_id: {}", e))
                })?;
                Ok(DiscordGuildDto {
                    id: g.id,
                    guild_id,
                    name: g.name,
                    icon_hash: g.icon_hash,
                })
            })
            .transpose()
    }
}
