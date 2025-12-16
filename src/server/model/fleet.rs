use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Parameters for creating a fleet
#[derive(Debug, Clone)]
pub struct CreateFleetParams {
    pub category_id: i32,
    pub name: String,
    pub commander_id: u64,
    pub fleet_time: DateTime<Utc>,
    pub description: Option<String>,
    pub field_values: HashMap<i32, String>,
    pub hidden: bool,
    pub disable_reminder: bool,
}

/// Parameters for updating a fleet
#[derive(Debug, Clone)]
pub struct UpdateFleetParams {
    pub id: i32,
    pub category_id: Option<i32>,
    pub name: Option<String>,
    pub fleet_time: Option<DateTime<Utc>>,
    pub description: Option<Option<String>>,
    pub field_values: Option<HashMap<i32, String>>,
    pub hidden: Option<bool>,
    pub disable_reminder: Option<bool>,
}
