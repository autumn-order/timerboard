use sea_orm::DbErr;

/// Represents a Discord guild member.
///
/// This param model represents a Discord user who is a member of a guild,
/// including their username and optional guild-specific nickname. It's used
/// by the service layer to work with guild members without depending on
/// database entity models.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuildMemberParam {
    /// Discord user ID as a u64.
    pub user_id: u64,
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Discord username.
    pub username: String,
    /// Optional guild-specific nickname.
    pub nickname: Option<String>,
}

impl DiscordGuildMemberParam {
    /// Converts an entity model to a param model at the repository boundary.
    ///
    /// Parses the string IDs from the database into u64 values for type-safe
    /// usage in the service layer. Also preserves username and nickname data.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuildMemberParam)` - Successfully converted param model
    /// - `Err(DbErr::Custom)` - Failed to parse user_id or guild_id as u64
    pub fn from_entity(entity: entity::discord_guild_member::Model) -> Result<Self, DbErr> {
        let user_id = entity
            .user_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse user_id: {}", e)))?;

        let guild_id = entity
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            user_id,
            guild_id,
            username: entity.username,
            nickname: entity.nickname,
        })
    }
}
