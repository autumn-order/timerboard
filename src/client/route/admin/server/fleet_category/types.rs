/// Tab selection for the configuration section
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ConfigTab {
    #[default]
    AccessRoles,
    PingRoles,
    Channels,
}

/// Role data
#[derive(Clone, PartialEq)]
pub struct RoleData {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub position: i16,
}

/// Channel data
#[derive(Clone, PartialEq)]
pub struct ChannelData {
    pub id: u64,
    pub name: String,
    pub position: i32,
}

/// Access role with permissions
#[derive(Clone, PartialEq)]
pub struct AccessRoleData {
    pub role: RoleData,
    pub can_view: bool,
    pub can_create: bool,
    pub can_manage: bool,
}

/// Form field values
#[derive(Clone, Default, PartialEq)]
pub struct FormFieldsData {
    pub category_name: String,
    pub ping_format_id: Option<i32>,
    pub search_query: String,
    pub ping_cooldown_str: String,
    pub ping_reminder_str: String,
    pub max_pre_ping_str: String,
    pub active_tab: ConfigTab,
    pub role_search_query: String,
    pub channel_search_query: String,
    pub access_roles: Vec<AccessRoleData>,
    pub ping_roles: Vec<RoleData>,
    pub channels: Vec<ChannelData>,
}

/// Validation errors for duration fields
#[derive(Clone, Default, PartialEq)]
pub struct ValidationErrorsData {
    pub ping_cooldown: Option<String>,
    pub ping_reminder: Option<String>,
    pub max_pre_ping: Option<String>,
}
