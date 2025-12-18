use sea_orm::DbErr;

/// Represents a user's Discord guild role membership.
///
/// This param model represents the relationship between a user and a Discord role
/// within a guild. It's used by the service layer to work with role memberships
/// without depending on database entity models.
#[derive(Debug, Clone, PartialEq)]
pub struct UserDiscordGuildRoleParam {
    /// Discord user ID as a u64.
    pub user_id: u64,
    /// Discord role ID as a u64.
    pub role_id: u64,
}

impl UserDiscordGuildRoleParam {
    /// Converts an entity model to a param model at the repository boundary.
    ///
    /// Parses the string IDs from the database into u64 values for type-safe
    /// usage in the service layer.
    ///
    /// # Arguments
    /// - `entity` - The database entity model to convert
    ///
    /// # Returns
    /// - `Ok(UserDiscordGuildRoleParam)` - Successfully converted param model
    /// - `Err(DbErr::Custom)` - Failed to parse user_id or role_id as u64
    pub fn from_entity(entity: entity::user_discord_guild_role::Model) -> Result<Self, DbErr> {
        let user_id = entity
            .user_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse user_id: {}", e)))?;

        let role_id = entity
            .role_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse role_id: {}", e)))?;

        Ok(Self { user_id, role_id })
    }
}
