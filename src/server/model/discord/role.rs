use sea_orm::DbErr;

/// Represents a Discord guild role.
///
/// This param model represents a role within a Discord guild with its display
/// properties (name, color, position). It's used by the service layer to work
/// with Discord roles without depending on database entity models.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscordGuildRoleParam {
    /// Discord role ID as a u64.
    pub role_id: u64,
    /// Discord guild ID as a u64.
    pub guild_id: u64,
    /// Role display name.
    pub name: String,
    /// Role color in hex format (e.g., "#FF5733").
    pub color: String,
    /// Role position in the guild's role hierarchy (higher = more important).
    pub position: i16,
}

impl DiscordGuildRoleParam {
    /// Converts an entity model to a param model at the repository boundary.
    ///
    /// Parses the string IDs from the database into u64 values for type-safe
    /// usage in the service layer. Also preserves role display properties.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(DiscordGuildRoleParam)` - Successfully converted param model
    /// - `Err(DbErr::Custom)` - Failed to parse role_id or guild_id as u64
    pub fn from_entity(entity: entity::discord_guild_role::Model) -> Result<Self, DbErr> {
        let role_id = entity
            .role_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse role_id: {}", e)))?;

        let guild_id = entity
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            role_id,
            guild_id,
            name: entity.name,
            color: entity.color,
            position: entity.position,
        })
    }
}
