//! Fleet notification service for Discord message management.
//!
//! This module provides the `FleetNotificationService` for managing Discord notifications
//! related to fleet events. It orchestrates fleet message posting, updates, and cancellations
//! across configured Discord channels with role pings and embedded fleet information.
//!
//! The service is organized into separate modules by concern:
//! - `builder` - Embed building utilities
//! - `creation` - Initial fleet creation notifications
//! - `reminder` - Fleet reminder notifications
//! - `formup` - Fleet formup (start) notifications
//! - `list` - Upcoming fleets list management

pub mod builder;
pub mod cancel;
pub mod creation;
pub mod formup;
pub mod list;
pub mod reminder;
pub mod update;

use sea_orm::DatabaseConnection;
use serenity::{all::CreateEmbed, http::Http};
use std::sync::Arc;

use crate::server::{
    data::{category::FleetCategoryRepository, ping_format::field::PingFormatFieldRepository},
    error::AppError,
    model::{category::FleetCategoryWithRelations, fleet::Fleet, ping_format::PingFormatField},
    util::parse::parse_u64_from_string,
};

/// Service providing Discord notification operations for fleet events.
///
/// This struct holds references to the database connection, Discord HTTP client, and
/// application URL. It provides methods for posting fleet notifications (creation,
/// reminders, formup), and maintaining an upcoming fleets list in configured channels.
///
/// The service layer contains business logic and coordinates between repositories
/// (data layer) and the Discord API. It does not perform direct database queries or
/// entity conversions - those responsibilities belong to the repository layer.
pub struct FleetNotificationService<'a> {
    /// Database connection for accessing fleet and notification data via repositories
    db: &'a DatabaseConnection,
    /// Discord HTTP client for sending and editing messages
    http: Arc<Http>,
    /// Base application URL for embedding links in notifications
    app_url: String,
}

impl<'a> FleetNotificationService<'a> {
    /// Creates a new FleetNotificationService instance.
    ///
    /// # Arguments
    /// - `db` - Reference to the database connection
    /// - `http` - Arc-wrapped Discord HTTP client for API requests
    /// - `app_url` - Base URL of the application for embedding in notifications
    ///
    /// # Returns
    /// - `FleetNotificationService` - New service instance
    pub fn new(db: &'a DatabaseConnection, http: Arc<Http>, app_url: String) -> Self {
        Self { db, http, app_url }
    }

    /// Fetches category data and parses guild_id
    ///
    /// # Arguments
    /// - `category_id` - The category ID to fetch
    ///
    /// # Returns
    /// - `Ok((category_data, guild_id))` - Category data and parsed guild ID
    /// - `Err(AppError::NotFound)` - Category not found
    /// - `Err(AppError::InternalError)` - Failed to parse guild ID
    async fn get_category_data_with_guild_id(
        &self,
        category_id: i32,
    ) -> Result<(FleetCategoryWithRelations, u64), AppError> {
        let category_repo = FleetCategoryRepository::new(self.db);
        let category_data = category_repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Fleet category not found".to_string()))?;

        let guild_id = parse_u64_from_string(category_data.category.guild_id.clone())?;
        Ok((category_data, guild_id))
    }

    /// Fetches ping format fields for a category
    ///
    /// # Arguments
    /// - `category_data` - Category data containing ping format
    /// - `guild_id` - Guild ID for the ping format
    ///
    /// # Returns
    /// - `Ok(fields)` - Vector of ping format fields
    /// - `Err(AppError::NotFound)` - Ping format not found
    /// - `Err(AppError::Database)` - Database error retrieving fields
    async fn get_ping_format_fields(
        &self,
        category_data: &FleetCategoryWithRelations,
        guild_id: u64,
    ) -> Result<Vec<PingFormatField>, AppError> {
        let ping_format_field_repo = PingFormatFieldRepository::new(self.db);
        let ping_format = category_data
            .ping_format
            .as_ref()
            .ok_or_else(|| AppError::NotFound("Ping format not found".to_string()))?;

        let fields = ping_format_field_repo
            .get_by_ping_format_id(guild_id, ping_format.id)
            .await?;

        Ok(fields)
    }

    /// Builds fleet embed with commander name fetching
    ///
    /// # Arguments
    /// - `fleet` - Fleet data
    /// - `fields` - Ping format fields
    /// - `field_values` - Field values map
    /// - `color` - Embed color
    /// - `guild_id` - Guild ID for fetching commander name
    ///
    /// # Returns
    /// - `Ok(embed)` - Built embed
    /// - `Err(AppError)` - Error building embed or fetching commander
    async fn build_fleet_embed_with_commander(
        &self,
        fleet: &Fleet,
        fields: &[PingFormatField],
        field_values: &std::collections::HashMap<i32, String>,
        color: u32,
        guild_id: u64,
    ) -> Result<CreateEmbed, AppError> {
        let commander_name =
            builder::get_commander_name(self.http.clone(), fleet, guild_id).await?;

        let embed = builder::build_fleet_embed(
            fleet,
            fields,
            field_values,
            color,
            &commander_name,
            &self.app_url,
        )
        .await?;

        Ok(embed)
    }

    /// Builds ping content with role mentions
    ///
    /// # Arguments
    /// - `title` - Title to prepend to the content
    /// - `category_data` - Category data containing ping roles
    /// - `guild_id` - Guild ID for @everyone detection
    ///
    /// # Returns
    /// - `Ok(content)` - Built content string with role pings
    /// - `Err(AppError::InternalError)` - Failed to parse role ID
    fn build_ping_content(
        &self,
        title: &str,
        category_data: &FleetCategoryWithRelations,
        guild_id: u64,
    ) -> Result<String, AppError> {
        let mut content = format!("{}\n\n", title);
        for (ping_role, _) in &category_data.ping_roles {
            let role_id = parse_u64_from_string(ping_role.role_id.clone())?;

            // @everyone role has the same ID as the guild - use @everyone instead of <@&guild_id>
            if role_id == guild_id {
                content.push_str("@everyone ");
            } else {
                content.push_str(&format!("<@&{}> ", role_id));
            }
        }

        Ok(content)
    }
}
