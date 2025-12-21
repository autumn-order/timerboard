//! Fleet notification service for Discord message management.
//!
//! This module provides the `FleetNotificationService` for managing Discord notifications
//! related to fleet events. It orchestrates fleet message posting, updates, and cancellations
//! across configured Discord channels with role pings and embedded fleet information. The
//! service also maintains an "upcoming fleets" list message that provides a centralized view
//! of scheduled events in each channel.

mod builder;
mod posting;
mod update;

use sea_orm::DatabaseConnection;
use serenity::http::Http;
use std::sync::Arc;

use crate::server::{error::AppError, model::fleet::Fleet};

use posting::FleetNotificationPosting;
use update::FleetNotificationUpdate;

/// Service providing Discord notification operations for fleet events.
///
/// This struct holds references to the database connection, Discord HTTP client, and
/// application URL. It provides methods for posting fleet notifications (creation,
/// reminders, formup), updating existing messages, cancelling fleets, and maintaining
/// an upcoming fleets list in configured channels.
pub struct FleetNotificationService<'a> {
    /// Database connection for accessing fleet and notification data
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

    /// Posts fleet creation message to all configured channels.
    ///
    /// Creates Discord messages with fleet details in all channels configured for the
    /// fleet's category. Only posts if the fleet is not hidden. Message IDs are stored
    /// in the database for later updates or cancellations. Uses blue embed color (0x3498db).
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted creation messages to all channels
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error storing message records
    pub async fn post_fleet_creation(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        let posting =
            FleetNotificationPosting::new(self.db, self.http.clone(), self.app_url.clone());
        posting.post_fleet_creation(fleet, field_values).await
    }

    /// Posts fleet reminder message as a reply to the creation message.
    ///
    /// Creates reminder notifications before fleet time to alert participants. If creation
    /// messages exist, replies to them. If the fleet was initially hidden (no creation
    /// messages), posts as new messages. Skips posting if `disable_reminder` is true.
    /// Uses orange embed color (0xf39c12).
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted reminder messages or skipped (if disabled)
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving or storing messages
    pub async fn post_fleet_reminder(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        let posting =
            FleetNotificationPosting::new(self.db, self.http.clone(), self.app_url.clone());
        posting.post_fleet_reminder(fleet, field_values).await
    }

    /// Posts fleet formup message as a reply to existing fleet messages.
    ///
    /// Creates formup notifications at fleet time to signal immediate gathering. Replies
    /// to the most recent existing message (reminder or creation) for each channel.
    /// Uses red embed color (0xe74c3c) to indicate urgency.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model containing event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted formup messages to all channels
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving or storing messages
    pub async fn post_fleet_formup(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        let posting =
            FleetNotificationPosting::new(self.db, self.http.clone(), self.app_url.clone());
        posting.post_fleet_formup(fleet, field_values).await
    }

    /// Updates all existing fleet messages with new fleet information.
    ///
    /// Edits all Discord messages associated with the fleet to reflect updated details.
    /// Continues updating remaining messages even if individual updates fail. Uses blue
    /// embed color (0x3498db) for updates. Logs errors for failed updates but doesn't
    /// propagate them to allow partial success.
    ///
    /// # Arguments
    /// - `fleet` - Updated fleet domain model with current event details
    /// - `field_values` - Map of field_id to value for custom ping format fields
    ///
    /// # Returns
    /// - `Ok(())` - Successfully updated all messages (or no messages exist)
    /// - `Err(AppError::NotFound)` - Fleet category or ping format not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving messages or fields
    pub async fn update_fleet_messages(
        &self,
        fleet: &Fleet,
        field_values: &std::collections::HashMap<i32, String>,
    ) -> Result<(), AppError> {
        let update = FleetNotificationUpdate::new(self.db, self.http.clone());
        update
            .update_fleet_messages(fleet, field_values, &self.app_url)
            .await
    }

    /// Cancels all existing fleet messages by editing them with cancellation notice.
    ///
    /// Edits all Discord messages associated with the fleet to display cancellation
    /// information. Uses gray embed color (0x95a5a6) and includes cancellation timestamp
    /// and cancelled-by information. Continues cancelling remaining messages even if
    /// individual edits fail.
    ///
    /// # Arguments
    /// - `fleet` - Fleet domain model being cancelled
    ///
    /// # Returns
    /// - `Ok(())` - Successfully cancelled all messages (or no messages exist)
    /// - `Err(AppError::NotFound)` - Fleet category not found
    /// - `Err(AppError::InternalError)` - Invalid ID format or timestamp
    /// - `Err(AppError::Database)` - Database error retrieving messages
    pub async fn cancel_fleet_messages(&self, fleet: &Fleet) -> Result<(), AppError> {
        let update = FleetNotificationUpdate::new(self.db, self.http.clone());
        update.cancel_fleet_messages(fleet).await
    }

    /// Posts or updates the upcoming fleets list for a channel.
    ///
    /// Creates or updates a single message displaying all upcoming non-hidden fleets
    /// for categories configured to post in the channel. The list includes links to
    /// fleet messages and relative timestamps. Intelligently edits the existing list
    /// if it's still the most recent message, or deletes and reposts if other messages
    /// have been sent since. Uses Discord blurple color (0x5865F2).
    ///
    /// # Arguments
    /// - `channel_id_str` - Discord channel ID as string
    ///
    /// # Returns
    /// - `Ok(())` - Successfully posted/updated the upcoming fleets list
    /// - `Err(AppError::InternalError)` - Invalid channel or message ID format
    /// - `Err(AppError::Database)` - Database error retrieving fleets or categories
    pub async fn post_upcoming_fleets_list(&self, channel_id: u64) -> Result<(), AppError> {
        let posting =
            FleetNotificationPosting::new(self.db, self.http.clone(), self.app_url.clone());
        posting.post_upcoming_fleets_list(channel_id).await
    }
}
