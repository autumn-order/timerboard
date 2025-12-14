use chrono::Duration;

/// Simple access role data without UI display fields (name, color)
#[derive(Debug, Clone)]
pub struct AccessRoleData {
    pub role_id: i64,
    pub can_view: bool,
    pub can_create: bool,
    pub can_manage: bool,
}

/// Access role with enriched display data
#[derive(Debug, Clone)]
pub struct AccessRole {
    pub role_id: i64,
    pub role_name: String,
    pub role_color: String,
    pub can_view: bool,
    pub can_create: bool,
    pub can_manage: bool,
}

impl AccessRole {
    pub fn from_entity(
        entity: entity::fleet_category_access_role::Model,
        role_model: Option<entity::discord_guild_role::Model>,
    ) -> Self {
        Self {
            role_id: entity.role_id,
            role_name: role_model
                .as_ref()
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Unknown Role ({})", entity.role_id)),
            role_color: role_model
                .as_ref()
                .map(|r| r.color.clone())
                .unwrap_or_else(|| "#99aab5".to_string()),
            can_view: entity.can_view,
            can_create: entity.can_create,
            can_manage: entity.can_manage,
        }
    }

    pub fn to_dto(&self) -> crate::model::category::FleetCategoryAccessRoleDto {
        crate::model::category::FleetCategoryAccessRoleDto {
            role_id: self.role_id,
            role_name: self.role_name.clone(),
            role_color: self.role_color.clone(),
            can_view: self.can_view,
            can_create: self.can_create,
            can_manage: self.can_manage,
        }
    }
}

impl From<crate::model::category::FleetCategoryAccessRoleDto> for AccessRoleData {
    fn from(dto: crate::model::category::FleetCategoryAccessRoleDto) -> Self {
        Self {
            role_id: dto.role_id,
            can_view: dto.can_view,
            can_create: dto.can_create,
            can_manage: dto.can_manage,
        }
    }
}

/// Ping role with enriched display data
#[derive(Debug, Clone)]
pub struct PingRole {
    pub role_id: i64,
    pub role_name: String,
    pub role_color: String,
}

impl PingRole {
    pub fn from_entity(
        entity: entity::fleet_category_ping_role::Model,
        role_model: Option<entity::discord_guild_role::Model>,
    ) -> Self {
        Self {
            role_id: entity.role_id,
            role_name: role_model
                .as_ref()
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Unknown Role ({})", entity.role_id)),
            role_color: role_model
                .as_ref()
                .map(|r| r.color.clone())
                .unwrap_or_else(|| "#99aab5".to_string()),
        }
    }

    pub fn to_dto(&self) -> crate::model::category::FleetCategoryPingRoleDto {
        crate::model::category::FleetCategoryPingRoleDto {
            role_id: self.role_id,
            role_name: self.role_name.clone(),
            role_color: self.role_color.clone(),
        }
    }
}

/// Channel with enriched display data
#[derive(Debug, Clone)]
pub struct Channel {
    pub channel_id: i64,
    pub channel_name: String,
}

impl Channel {
    pub fn from_entity(
        entity: entity::fleet_category_channel::Model,
        channel_model: Option<entity::discord_guild_channel::Model>,
    ) -> Self {
        Self {
            channel_id: entity.channel_id,
            channel_name: channel_model
                .as_ref()
                .map(|ch| ch.name.clone())
                .unwrap_or_else(|| format!("Unknown Channel ({})", entity.channel_id)),
        }
    }

    pub fn to_dto(&self) -> crate::model::category::FleetCategoryChannelDto {
        crate::model::category::FleetCategoryChannelDto {
            channel_id: self.channel_id,
            channel_name: self.channel_name.clone(),
        }
    }
}

/// Parameters for creating a fleet category
#[derive(Debug, Clone)]
pub struct CreateFleetCategoryParams {
    pub guild_id: i64,
    pub ping_format_id: i32,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<AccessRoleData>,
    pub ping_roles: Vec<i64>,
    pub channels: Vec<i64>,
}

impl CreateFleetCategoryParams {
    pub fn from_dto(guild_id: i64, dto: crate::model::category::CreateFleetCategoryDto) -> Self {
        Self {
            guild_id,
            ping_format_id: dto.ping_format_id,
            name: dto.name,
            ping_lead_time: dto.ping_lead_time,
            ping_reminder: dto.ping_reminder,
            max_pre_ping: dto.max_pre_ping,
            access_roles: dto.access_roles.into_iter().map(Into::into).collect(),
            ping_roles: dto.ping_roles.into_iter().map(|pr| pr.role_id).collect(),
            channels: dto.channels.into_iter().map(|c| c.channel_id).collect(),
        }
    }
}

/// Parameters for updating a fleet category
#[derive(Debug, Clone)]
pub struct UpdateFleetCategoryParams {
    pub id: i32,
    pub guild_id: i64,
    pub ping_format_id: i32,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<AccessRoleData>,
    pub ping_roles: Vec<i64>,
    pub channels: Vec<i64>,
}

impl UpdateFleetCategoryParams {
    pub fn from_dto(
        id: i32,
        guild_id: i64,
        dto: crate::model::category::UpdateFleetCategoryDto,
    ) -> Self {
        Self {
            id,
            guild_id,
            ping_format_id: dto.ping_format_id,
            name: dto.name,
            ping_lead_time: dto.ping_lead_time,
            ping_reminder: dto.ping_reminder,
            max_pre_ping: dto.max_pre_ping,
            access_roles: dto.access_roles.into_iter().map(Into::into).collect(),
            ping_roles: dto.ping_roles.into_iter().map(|pr| pr.role_id).collect(),
            channels: dto.channels.into_iter().map(|c| c.channel_id).collect(),
        }
    }
}

/// Result from repository when fetching a category with related data
#[derive(Debug, Clone)]
pub struct FleetCategoryWithRelations {
    pub category: entity::fleet_category::Model,
    pub ping_format: Option<entity::ping_format::Model>,
    pub access_roles: Vec<(
        entity::fleet_category_access_role::Model,
        Option<entity::discord_guild_role::Model>,
    )>,
    pub ping_roles: Vec<(
        entity::fleet_category_ping_role::Model,
        Option<entity::discord_guild_role::Model>,
    )>,
    pub channels: Vec<(
        entity::fleet_category_channel::Model,
        Option<entity::discord_guild_channel::Model>,
    )>,
}

/// Result from repository when fetching category with basic info and ping format
#[derive(Debug, Clone)]
pub struct FleetCategoryWithFormat {
    pub category: entity::fleet_category::Model,
    pub ping_format: Option<entity::ping_format::Model>,
}

/// Result from repository when fetching paginated categories with counts
#[derive(Debug, Clone)]
pub struct FleetCategoryWithCounts {
    pub category: entity::fleet_category::Model,
    pub ping_format: Option<entity::ping_format::Model>,
    pub access_roles_count: usize,
    pub ping_roles_count: usize,
    pub channels_count: usize,
}

/// Full fleet category with all related data
#[derive(Debug, Clone)]
pub struct FleetCategory {
    pub id: i32,
    pub guild_id: i64,
    pub ping_format_id: i32,
    pub ping_format_name: String,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<AccessRole>,
    pub ping_roles: Vec<PingRole>,
    pub channels: Vec<Channel>,
}

impl FleetCategory {
    pub fn from_with_relations(data: FleetCategoryWithRelations) -> Self {
        let access_roles = data
            .access_roles
            .into_iter()
            .map(|(ar, role_model)| AccessRole::from_entity(ar, role_model))
            .collect();

        let ping_roles = data
            .ping_roles
            .into_iter()
            .map(|(pr, role_model)| PingRole::from_entity(pr, role_model))
            .collect();

        let channels = data
            .channels
            .into_iter()
            .map(|(c, channel_model)| Channel::from_entity(c, channel_model))
            .collect();

        Self {
            id: data.category.id,
            guild_id: data.category.guild_id,
            ping_format_id: data.category.ping_format_id,
            ping_format_name: data
                .ping_format
                .map(|pf| pf.name)
                .unwrap_or_else(|| "Unknown".to_string()),
            name: data.category.name,
            ping_lead_time: data
                .category
                .ping_cooldown
                .map(|s| Duration::seconds(s as i64)),
            ping_reminder: data
                .category
                .ping_reminder
                .map(|s| Duration::seconds(s as i64)),
            max_pre_ping: data
                .category
                .max_pre_ping
                .map(|s| Duration::seconds(s as i64)),
            access_roles,
            ping_roles,
            channels,
        }
    }

    pub fn to_dto(self) -> crate::model::category::FleetCategoryDto {
        crate::model::category::FleetCategoryDto {
            id: self.id,
            guild_id: self.guild_id,
            ping_format_id: self.ping_format_id,
            ping_format_name: self.ping_format_name,
            name: self.name,
            ping_lead_time: self.ping_lead_time,
            ping_reminder: self.ping_reminder,
            max_pre_ping: self.max_pre_ping,
            access_roles: self
                .access_roles
                .into_iter()
                .map(|ar| ar.to_dto())
                .collect(),
            ping_roles: self.ping_roles.into_iter().map(|pr| pr.to_dto()).collect(),
            channels: self.channels.into_iter().map(|c| c.to_dto()).collect(),
        }
    }
}

/// Fleet category list item with counts
#[derive(Debug, Clone)]
pub struct FleetCategoryListItem {
    pub id: i32,
    pub guild_id: i64,
    pub ping_format_id: i32,
    pub ping_format_name: String,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles_count: usize,
    pub ping_roles_count: usize,
    pub channels_count: usize,
}

impl FleetCategoryListItem {
    pub fn from_with_counts(data: FleetCategoryWithCounts) -> Self {
        Self {
            id: data.category.id,
            guild_id: data.category.guild_id,
            ping_format_id: data.category.ping_format_id,
            ping_format_name: data
                .ping_format
                .map(|pf| pf.name)
                .unwrap_or_else(|| "Unknown".to_string()),
            name: data.category.name,
            ping_lead_time: data
                .category
                .ping_cooldown
                .map(|s| Duration::seconds(s as i64)),
            ping_reminder: data
                .category
                .ping_reminder
                .map(|s| Duration::seconds(s as i64)),
            max_pre_ping: data
                .category
                .max_pre_ping
                .map(|s| Duration::seconds(s as i64)),
            access_roles_count: data.access_roles_count,
            ping_roles_count: data.ping_roles_count,
            channels_count: data.channels_count,
        }
    }

    pub fn from_entity(category: entity::fleet_category::Model) -> Self {
        Self {
            id: category.id,
            guild_id: category.guild_id,
            ping_format_id: category.ping_format_id,
            ping_format_name: String::new(),
            name: category.name,
            ping_lead_time: category.ping_cooldown.map(|s| Duration::seconds(s as i64)),
            ping_reminder: category.ping_reminder.map(|s| Duration::seconds(s as i64)),
            max_pre_ping: category.max_pre_ping.map(|s| Duration::seconds(s as i64)),
            access_roles_count: 0,
            ping_roles_count: 0,
            channels_count: 0,
        }
    }

    pub fn to_dto(self) -> crate::model::category::FleetCategoryListItemDto {
        crate::model::category::FleetCategoryListItemDto {
            id: self.id,
            guild_id: self.guild_id,
            ping_format_id: self.ping_format_id,
            ping_format_name: self.ping_format_name,
            name: self.name,
            ping_lead_time: self.ping_lead_time,
            ping_reminder: self.ping_reminder,
            max_pre_ping: self.max_pre_ping,
            access_roles_count: self.access_roles_count,
            ping_roles_count: self.ping_roles_count,
            channels_count: self.channels_count,
        }
    }
}

/// Paginated fleet categories result
#[derive(Debug, Clone)]
pub struct PaginatedFleetCategories {
    pub categories: Vec<FleetCategoryListItem>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl PaginatedFleetCategories {
    pub fn to_dto(self) -> crate::model::category::PaginatedFleetCategoriesDto {
        crate::model::category::PaginatedFleetCategoriesDto {
            categories: self.categories.into_iter().map(|c| c.to_dto()).collect(),
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}
