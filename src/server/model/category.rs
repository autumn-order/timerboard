//! Fleet category domain models and parameters.
//!
//! Provides domain models for fleet categories, which organize fleet operations with
//! associated ping formats, access controls, notification roles, and channels. Includes
//! parameter types for create/update operations and models for different query contexts.

use chrono::Duration;
use sea_orm::DbErr;

/// Access role permissions without display properties.
///
/// Contains only permission flags for a role. Used when display information
/// (name, color) is not needed or unavailable.
#[derive(Debug, Clone)]
pub struct AccessRoleData {
    /// Discord role ID as a u64.
    pub role_id: u64,
    /// Whether the role can view fleets in this category.
    pub can_view: bool,
    /// Whether the role can create fleets in this category.
    pub can_create: bool,
    /// Whether the role can manage fleets in this category.
    pub can_manage: bool,
}

/// Access role with permissions and display properties.
///
/// Includes permission flags along with enriched display data from the guild role
/// (name, color, position) for UI presentation.
#[derive(Debug, Clone)]
pub struct AccessRole {
    /// Discord role ID as a u64.
    pub role_id: u64,
    /// Role display name.
    pub role_name: String,
    /// Role color in hex format (e.g., "#FF5733").
    pub role_color: String,
    /// Role position in guild hierarchy.
    pub position: i16,
    /// Whether the role can view fleets in this category.
    pub can_view: bool,
    /// Whether the role can create fleets in this category.
    pub can_create: bool,
    /// Whether the role can manage fleets in this category.
    pub can_manage: bool,
}

impl AccessRole {
    /// Converts entity models to a domain model at the repository boundary.
    ///
    /// Enriches access role permissions with display properties from the guild role.
    /// Falls back to defaults if role model is unavailable.
    ///
    /// # Arguments
    /// - `entity` - The fleet category access role entity from the database
    /// - `role_model` - Optional guild role entity for display properties
    ///
    /// # Returns
    /// - `Ok(AccessRole)` - Successfully converted domain model with enriched data
    /// - `Err(DbErr::Custom)` - Failed to parse role_id as u64
    pub fn from_entity(
        entity: entity::fleet_category_access_role::Model,
        role_model: Option<entity::discord_guild_role::Model>,
    ) -> Result<Self, DbErr> {
        let role_id = entity
            .role_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse role_id: {}", e)))?;

        Ok(Self {
            role_id,
            role_name: role_model
                .as_ref()
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Unknown Role ({})", role_id)),
            role_color: role_model
                .as_ref()
                .map(|r| r.color.clone())
                .unwrap_or_else(|| "#99aab5".to_string()),
            position: role_model.as_ref().map(|r| r.position).unwrap_or(0),
            can_view: entity.can_view,
            can_create: entity.can_create,
            can_manage: entity.can_manage,
        })
    }

    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `FleetCategoryAccessRoleDto` - DTO with all access role fields for serialization
    pub fn into_dto(self) -> crate::model::category::FleetCategoryAccessRoleDto {
        crate::model::category::FleetCategoryAccessRoleDto {
            role_id: self.role_id,
            role_name: self.role_name.clone(),
            role_color: self.role_color.clone(),
            position: self.position,
            can_view: self.can_view,
            can_create: self.can_create,
            can_manage: self.can_manage,
        }
    }
}

impl From<crate::model::category::FleetCategoryAccessRoleDto> for AccessRoleData {
    /// Converts a DTO to access role data for service layer operations.
    ///
    /// Extracts only the permission flags, discarding display properties.
    fn from(dto: crate::model::category::FleetCategoryAccessRoleDto) -> Self {
        Self {
            role_id: dto.role_id,
            can_view: dto.can_view,
            can_create: dto.can_create,
            can_manage: dto.can_manage,
        }
    }
}

/// Ping role with display properties for notification targeting.
///
/// Represents a role that will be mentioned in fleet ping messages, with enriched
/// display data for UI presentation.
#[derive(Debug, Clone)]
pub struct PingRole {
    /// Discord role ID as a u64.
    pub role_id: u64,
    /// Role display name.
    pub role_name: String,
    /// Role color in hex format (e.g., "#FF5733").
    pub role_color: String,
    /// Role position in guild hierarchy.
    pub position: i16,
}

impl PingRole {
    /// Converts entity models to a domain model at the repository boundary.
    ///
    /// Enriches ping role data with display properties from the guild role.
    /// Falls back to defaults if role model is unavailable.
    ///
    /// # Arguments
    /// - `entity` - The fleet category ping role entity from the database
    /// - `role_model` - Optional guild role entity for display properties
    ///
    /// # Returns
    /// - `Ok(PingRole)` - Successfully converted domain model with enriched data
    /// - `Err(DbErr::Custom)` - Failed to parse role_id as u64
    pub fn from_entity(
        entity: entity::fleet_category_ping_role::Model,
        role_model: Option<entity::discord_guild_role::Model>,
    ) -> Result<Self, DbErr> {
        let role_id = entity
            .role_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse role_id: {}", e)))?;

        Ok(Self {
            role_id,
            role_name: role_model
                .as_ref()
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Unknown Role ({})", role_id)),
            role_color: role_model
                .as_ref()
                .map(|r| r.color.clone())
                .unwrap_or_else(|| "#99aab5".to_string()),
            position: role_model.as_ref().map(|r| r.position).unwrap_or(0),
        })
    }

    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `FleetCategoryPingRoleDto` - DTO with all ping role fields for serialization
    pub fn into_dto(self) -> crate::model::category::FleetCategoryPingRoleDto {
        crate::model::category::FleetCategoryPingRoleDto {
            role_id: self.role_id,
            role_name: self.role_name.clone(),
            role_color: self.role_color.clone(),
            position: self.position,
        }
    }
}

/// Discord channel with display properties for fleet list posting.
///
/// Represents a channel where fleet lists will be posted, with enriched display
/// data for UI presentation.
#[derive(Debug, Clone)]
pub struct Channel {
    /// Discord channel ID as a u64.
    pub channel_id: u64,
    /// Channel display name.
    pub channel_name: String,
    /// Channel position in guild's channel list.
    pub position: i32,
}

impl Channel {
    /// Converts entity models to a domain model at the repository boundary.
    ///
    /// Enriches channel association with display properties from the guild channel.
    /// Falls back to defaults if channel model is unavailable.
    ///
    /// # Arguments
    /// - `entity` - The fleet category channel entity from the database
    /// - `channel_model` - Optional guild channel entity for display properties
    ///
    /// # Returns
    /// - `Ok(Channel)` - Successfully converted domain model with enriched data
    /// - `Err(DbErr::Custom)` - Failed to parse channel_id as u64
    pub fn from_entity(
        entity: entity::fleet_category_channel::Model,
        channel_model: Option<entity::discord_guild_channel::Model>,
    ) -> Result<Self, DbErr> {
        let channel_id = entity
            .channel_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse channel_id: {}", e)))?;

        Ok(Self {
            channel_id,
            channel_name: channel_model
                .as_ref()
                .map(|ch| ch.name.clone())
                .unwrap_or_else(|| format!("Unknown Channel ({})", channel_id)),
            position: channel_model.as_ref().map(|ch| ch.position).unwrap_or(0),
        })
    }

    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `FleetCategoryChannelDto` - DTO with all channel fields for serialization
    pub fn into_dto(self) -> crate::model::category::FleetCategoryChannelDto {
        crate::model::category::FleetCategoryChannelDto {
            channel_id: self.channel_id,
            channel_name: self.channel_name.clone(),
            position: self.position,
        }
    }
}

/// Parameters for creating a new fleet category with role/channel associations.
///
/// Includes all configuration for the category including ping format, timing settings,
/// and the initial set of access roles, ping roles, and channels.
#[derive(Debug, Clone)]
pub struct CreateFleetCategoryParams {
    pub guild_id: u64,
    pub ping_format_id: i32,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<AccessRoleData>,
    pub ping_roles: Vec<u64>,
    pub channels: Vec<u64>,
}

impl CreateFleetCategoryParams {
    pub fn from_dto(guild_id: u64, dto: crate::model::category::CreateFleetCategoryDto) -> Self {
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

/// Parameters for updating an existing fleet category and its associations.
///
/// Replaces all category configuration including role and channel associations.
/// Any existing associations not included will be removed.
#[derive(Debug, Clone)]
pub struct UpdateFleetCategoryParams {
    pub id: i32,
    pub guild_id: u64,
    pub ping_format_id: i32,
    pub name: String,
    pub ping_lead_time: Option<Duration>,
    pub ping_reminder: Option<Duration>,
    pub max_pre_ping: Option<Duration>,
    pub access_roles: Vec<AccessRoleData>,
    pub ping_roles: Vec<u64>,
    pub channels: Vec<u64>,
}

impl UpdateFleetCategoryParams {
    pub fn from_dto(
        id: i32,
        guild_id: u64,
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

/// Fleet category with all related entity models for conversion.
///
/// Raw repository result containing the category and all related entities with
/// optional guild role/channel data for enrichment.
#[derive(Debug, Clone)]
/// Fleet category with all related entity data loaded.
///
/// Used internally to hold entity models before conversion to domain models.
/// Contains the category, ping format, and all related roles and channels with
/// their display properties.
pub struct FleetCategoryWithRelations {
    /// The fleet category entity.
    pub category: entity::fleet_category::Model,
    /// The ping format entity if available.
    pub ping_format: Option<entity::ping_format::Model>,
    /// Access roles with their guild role entities for display properties.
    pub access_roles: Vec<(
        entity::fleet_category_access_role::Model,
        Option<entity::discord_guild_role::Model>,
    )>,
    /// Ping roles with their guild role entities for display properties.
    pub ping_roles: Vec<(
        entity::fleet_category_ping_role::Model,
        Option<entity::discord_guild_role::Model>,
    )>,
    /// Channels with their guild channel entities for display properties.
    pub channels: Vec<(
        entity::fleet_category_channel::Model,
        Option<entity::discord_guild_channel::Model>,
    )>,
}

/// Fleet category with relationship counts for list display.
///
/// Repository result for paginated category listings, including counts of
/// associated roles and channels without loading full relationship data.
#[derive(Debug, Clone)]
/// Fleet category with aggregate counts instead of full related data.
///
/// Used for list views where only counts are needed, avoiding expensive
/// relationship loading. Contains the category, ping format, and counts
/// of related entities.
pub struct FleetCategoryWithCounts {
    /// The fleet category entity.
    pub category: entity::fleet_category::Model,
    /// The ping format entity if available.
    pub ping_format: Option<entity::ping_format::Model>,
    /// Count of access roles for this category.
    pub access_roles_count: usize,
    /// Count of ping roles for this category.
    pub ping_roles_count: usize,
    /// Count of channels for this category.
    pub channels_count: usize,
}

/// Fleet category with complete configuration and enriched relationships.
///
/// Contains all category settings, timing configurations, and fully enriched
/// access roles, ping roles, and channels with display properties.
#[derive(Debug, Clone)]
pub struct FleetCategory {
    pub id: i32,
    pub guild_id: u64,
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
    /// Converts entity models with relations to a domain model.
    ///
    /// Transforms all related entity models to their domain representations,
    /// enriching with display properties where available.
    ///
    /// # Arguments
    /// - `data` - Fleet category with all related entity data
    ///
    /// # Returns
    /// - `Ok(FleetCategory)` - Successfully converted domain model with all relations
    /// - `Err(DbErr::Custom)` - Failed to parse IDs or convert related entities
    pub fn from_with_relations(data: FleetCategoryWithRelations) -> Result<Self, DbErr> {
        let guild_id = data
            .category
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        let access_roles: Result<Vec<AccessRole>, DbErr> = data
            .access_roles
            .into_iter()
            .map(|(ar, role_model)| AccessRole::from_entity(ar, role_model))
            .collect();

        let ping_roles: Result<Vec<PingRole>, DbErr> = data
            .ping_roles
            .into_iter()
            .map(|(pr, role_model)| PingRole::from_entity(pr, role_model))
            .collect();

        let channels: Result<Vec<Channel>, DbErr> = data
            .channels
            .into_iter()
            .map(|(c, channel_model)| Channel::from_entity(c, channel_model))
            .collect();

        Ok(Self {
            id: data.category.id,
            guild_id,
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
            access_roles: access_roles?,
            ping_roles: ping_roles?,
            channels: channels?,
        })
    }

    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `FleetCategoryDto` - DTO with all fleet category fields and relations for serialization
    pub fn into_dto(self) -> crate::model::category::FleetCategoryDto {
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
                .map(|ar| ar.into_dto())
                .collect(),
            ping_roles: self
                .ping_roles
                .into_iter()
                .map(|pr| pr.into_dto())
                .collect(),
            channels: self.channels.into_iter().map(|c| c.into_dto()).collect(),
        }
    }
}

/// Fleet category summary for paginated list display.
///
/// Contains category settings and relationship counts without loading full
/// relationship data, optimized for list views.
#[derive(Debug, Clone)]
pub struct FleetCategoryListItem {
    pub id: i32,
    pub guild_id: u64,
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
    /// Converts entity model with counts to a domain model for list views.
    ///
    /// # Arguments
    /// - `data` - Fleet category entity with aggregate counts
    ///
    /// # Returns
    /// - `Ok(FleetCategoryListItem)` - Successfully converted list item domain model
    /// - `Err(DbErr::Custom)` - Failed to parse guild_id as u64
    pub fn from_with_counts(data: FleetCategoryWithCounts) -> Result<Self, DbErr> {
        let guild_id = data
            .category
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            id: data.category.id,
            guild_id,
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
        })
    }

    pub fn from_entity(category: entity::fleet_category::Model) -> Result<Self, DbErr> {
        let guild_id = category
            .guild_id
            .parse::<u64>()
            .map_err(|e| DbErr::Custom(format!("Failed to parse guild_id: {}", e)))?;

        Ok(Self {
            id: category.id,
            guild_id,
            ping_format_id: category.ping_format_id,
            ping_format_name: String::new(),
            name: category.name,
            ping_lead_time: category.ping_cooldown.map(|s| Duration::seconds(s as i64)),
            ping_reminder: category.ping_reminder.map(|s| Duration::seconds(s as i64)),
            max_pre_ping: category.max_pre_ping.map(|s| Duration::seconds(s as i64)),
            access_roles_count: 0,
            ping_roles_count: 0,
            channels_count: 0,
        })
    }

    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `FleetCategoryListItemDto` - DTO with fleet category summary and counts for serialization
    pub fn into_dto(self) -> crate::model::category::FleetCategoryListItemDto {
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

/// Paginated result containing fleet category list items and metadata.
///
/// Includes the category list along with pagination information for UI display.
#[derive(Debug, Clone)]
/// Paginated list of fleet categories with metadata.
///
/// Contains a page of fleet category list items along with pagination metadata
/// for building paginated UI views.
pub struct PaginatedFleetCategories {
    /// Fleet category list items for the current page.
    pub categories: Vec<FleetCategoryListItem>,
    /// Total number of fleet categories across all pages.
    pub total: u64,
    /// Current page number (0-indexed).
    pub page: u64,
    /// Number of items per page.
    pub per_page: u64,
    /// Total number of pages available.
    pub total_pages: u64,
}

impl PaginatedFleetCategories {
    /// Converts domain model to DTO for API responses.
    ///
    /// # Returns
    /// - `PaginatedFleetCategoriesDto` - DTO with paginated categories and metadata for serialization
    pub fn into_dto(self) -> crate::model::category::PaginatedFleetCategoriesDto {
        crate::model::category::PaginatedFleetCategoriesDto {
            categories: self.categories.into_iter().map(|c| c.into_dto()).collect(),
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}
