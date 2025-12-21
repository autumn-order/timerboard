//! Fleet notification scheduler for automated Discord notifications.
//!
//! This module provides automated scheduling for fleet-related Discord notifications including:
//! - Reminder notifications sent before fleet time based on category configuration
//! - Form-up notifications sent when fleet time arrives
//! - Hourly updates to upcoming fleets list messages in configured channels
//!
//! The scheduler runs two primary jobs:
//! 1. Every minute: Check for fleets needing reminders or form-up notifications
//! 2. Every hour: Update upcoming fleets list messages in all configured channels

use chrono::{DateTime, Duration, Utc};
use dioxus_logger::tracing;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::http::Http;
use std::{collections::HashMap, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::server::{
    error::AppError, model::fleet::Fleet, service::fleet_notification::FleetNotificationService,
};

use super::sync::process_guild_sync;

/// Maximum age for sending form-up notifications.
///
/// Form-up notifications will only be sent if the fleet time is within this duration
/// from the current time. This prevents sending form-ups for very old fleets that
/// may have been missed during downtime.
static FORMUP_MAX_AGE: i64 = 5;

/// Starts the fleet notification scheduler.
///
/// Initializes and starts two cron jobs:
/// - Notifications job (every minute): Processes reminder and form-up notifications
/// - List update job (every hour): Updates upcoming fleets list messages
///
/// The scheduler continues running until the application shuts down.
///
/// # Arguments
/// - `db` - Database connection for querying fleet and notification data
/// - `discord_http` - Discord HTTP client for sending messages and embeds
/// - `app_url` - Application base URL for generating fleet detail links in embeds
///
/// # Returns
/// - `Ok(())` - Scheduler started successfully and is running
/// - `Err(AppError::Scheduler(_))` - Failed to create or start the scheduler
pub async fn start_scheduler(
    db: DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
) -> Result<(), AppError> {
    let scheduler = JobScheduler::new().await?;

    // Clone resources for the notifications job
    let job_db = db.clone();
    let job_http = discord_http.clone();
    let job_app_url = app_url.clone();

    // Schedule job to run every minute for reminders and form-ups
    let notifications_job = Job::new_async("0 * * * * *", move |_uuid, _lock| {
        let db = job_db.clone();
        let http = job_http.clone();
        let app_url = job_app_url.clone();

        Box::pin(async move {
            tracing::trace!("Running fleet notifications job");
            if let Err(e) = process_fleet_notifications(&db, http, app_url).await {
                tracing::error!("Error processing fleet notifications: {}", e);
            }
        })
    })?;

    scheduler.add(notifications_job).await?;

    // Clone resources for the list update job
    let list_db = db.clone();
    let list_http = discord_http.clone();
    let list_app_url = app_url.clone();

    // Schedule job to run every 30 minutes for upcoming fleets lists
    let list_job = Job::new_async("30 * * * * *", move |_uuid, _lock| {
        let db = list_db.clone();
        let http = list_http.clone();
        let app_url = list_app_url.clone();

        Box::pin(async move {
            tracing::trace!("Running upcoming fleets list update job");
            if let Err(e) = process_upcoming_fleets_lists(&db, http, app_url).await {
                tracing::error!("Error processing upcoming fleets lists: {}", e);
            }
        })
    })?;

    scheduler.add(list_job).await?;

    let sync_db = db.clone();
    let sync_http = discord_http.clone();

    let sync_job = Job::new_async("5 * * * * *", move |_uuid, _lock| {
        let db = sync_db.clone();
        let http = sync_http.clone();

        Box::pin(async move {
            tracing::trace!("Running periodic Discord sync update job");
            if let Err(e) = process_guild_sync(&db, http).await {
                tracing::error!("Error processing periodic Discord guild sync: {}", e)
            }
        })
    })?;

    scheduler.add(sync_job).await?;
    scheduler.start().await?;

    tracing::info!("Fleet notification scheduler started successfully");

    Ok(())
}

/// Processes fleet notifications for reminders and form-ups.
///
/// This function is called every minute by the scheduler and delegates to:
/// - `process_reminders` - Sends reminder notifications for fleets approaching their fleet time
/// - `process_formups` - Sends form-up notifications for fleets at their fleet time
///
/// Errors from individual notification types are logged but don't prevent processing
/// of other notification types.
///
/// # Arguments
/// - `db` - Database connection for querying fleet data
/// - `discord_http` - Discord HTTP client for sending notifications
/// - `app_url` - Application URL for embed links
///
/// # Returns
/// - `Ok(())` - All notification processing completed (individual errors are logged)
async fn process_fleet_notifications(
    db: &DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
) -> Result<(), AppError> {
    let now = Utc::now();

    // Process reminders
    if let Err(e) = process_reminders(db, discord_http.clone(), app_url.clone(), now).await {
        tracing::error!("Error processing reminders: {}", e);
    }

    // Process form-ups
    if let Err(e) = process_formups(db, discord_http, app_url, now).await {
        tracing::error!("Error processing form-ups: {}", e);
    }

    Ok(())
}

/// Processes fleets needing reminder notifications.
///
/// Queries the database for fleets that meet all reminder criteria:
/// - Not hidden
/// - Reminders not disabled for the fleet
/// - Category has a reminder time configured
/// - Current time is past the reminder time (fleet_time - category.ping_reminder)
/// - Current time is before fleet time (not yet formed up)
/// - No reminder notification has been sent yet
///
/// For each qualifying fleet, sends a reminder notification via the notification service.
///
/// # Arguments
/// - `db` - Database connection for querying fleet and category data
/// - `discord_http` - Discord HTTP client for sending reminder messages
/// - `app_url` - Application URL for generating fleet detail links
/// - `now` - Current UTC timestamp for calculating reminder times
///
/// # Returns
/// - `Ok(())` - All reminders processed (individual send failures are logged)
/// - `Err(DbErr(_))` - Database query failed
async fn process_reminders(
    db: &DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    // Query fleets that might need reminders
    let fleets = entity::prelude::Fleet::find()
        .filter(entity::fleet::Column::Hidden.eq(false))
        .filter(entity::fleet::Column::DisableReminder.eq(false))
        .all(db)
        .await?;

    tracing::debug!("Checking {} fleets for reminders", fleets.len());

    for fleet in fleets {
        // Get category to check ping_reminder
        let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
            .one(db)
            .await?;

        if let Some(category) = category {
            if let Some(reminder_seconds) = category.ping_reminder {
                // Calculate reminder time
                let reminder_duration = Duration::seconds(reminder_seconds as i64);
                let reminder_time = fleet.fleet_time - reminder_duration;

                // Check if reminder time has passed but fleet time hasn't
                if now >= reminder_time && now < fleet.fleet_time {
                    // Check if reminder already sent
                    let existing_reminder = entity::prelude::FleetMessage::find()
                        .filter(entity::fleet_message::Column::FleetId.eq(fleet.id))
                        .filter(entity::fleet_message::Column::MessageType.eq("reminder"))
                        .one(db)
                        .await?;

                    if existing_reminder.is_none() {
                        tracing::debug!(
                            "Sending reminder for fleet {} ({}) scheduled for {}",
                            fleet.id,
                            fleet.name,
                            fleet.fleet_time
                        );

                        let notification_service = FleetNotificationService::new(
                            db,
                            discord_http.clone(),
                            app_url.clone(),
                        );

                        // Get field values for the fleet
                        let field_values = entity::prelude::FleetFieldValue::find()
                            .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
                            .all(db)
                            .await?;

                        let field_values_map: HashMap<i32, String> = field_values
                            .into_iter()
                            .map(|fv| (fv.field_id, fv.value))
                            .collect();

                        let fleet_param = Fleet::from_entity(fleet.clone());

                        if let Err(e) = notification_service
                            .post_fleet_reminder(&fleet_param, &field_values_map)
                            .await
                        {
                            tracing::error!(
                                "Failed to send reminder for fleet {} ({}): {}",
                                fleet.id,
                                fleet.name,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Processes fleets needing form-up notifications.
///
/// Queries the database for fleets that meet form-up criteria:
/// - Fleet time has passed
/// - No form-up notification has been sent yet
/// - Fleet time is within the maximum age window (prevents very old fleets)
///
/// For each qualifying fleet, sends a form-up notification via the notification service.
///
/// # Arguments
/// - `db` - Database connection for querying fleet data
/// - `discord_http` - Discord HTTP client for sending form-up messages
/// - `app_url` - Application URL for generating fleet detail links
/// - `now` - Current UTC timestamp for checking fleet time and age
///
/// # Returns
/// - `Ok(())` - All form-ups processed (individual send failures are logged)
/// - `Err(DbErr(_))` - Database query failed
async fn process_formups(
    db: &DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    // Query fleets that might need form-up notifications
    let fleets = entity::prelude::Fleet::find()
        .filter(entity::fleet::Column::FleetTime.lte(now))
        .all(db)
        .await?;

    tracing::debug!("Checking {} fleets for form-ups", fleets.len());

    for fleet in fleets {
        // Check if form-up already sent
        let existing_formup = entity::prelude::FleetMessage::find()
            .filter(entity::fleet_message::Column::FleetId.eq(fleet.id))
            .filter(entity::fleet_message::Column::MessageType.eq("formup"))
            .one(db)
            .await?;

        if existing_formup.is_none() {
            // Only send form-up if fleet time is within the maximum age
            // This prevents sending form-ups for very old fleets
            let max_age = now - Duration::minutes(FORMUP_MAX_AGE);

            if fleet.fleet_time >= max_age {
                tracing::debug!(
                    "Sending form-up for fleet {} ({}) scheduled for {}",
                    fleet.id,
                    fleet.name,
                    fleet.fleet_time
                );

                let notification_service =
                    FleetNotificationService::new(db, discord_http.clone(), app_url.clone());

                // Get field values for the fleet
                let field_values = entity::prelude::FleetFieldValue::find()
                    .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
                    .all(db)
                    .await?;

                let field_values_map: HashMap<i32, String> = field_values
                    .into_iter()
                    .map(|fv| (fv.field_id, fv.value))
                    .collect();

                let fleet_param = Fleet::from_entity(fleet.clone());

                if let Err(e) = notification_service
                    .post_fleet_formup(&fleet_param, &field_values_map)
                    .await
                {
                    tracing::error!(
                        "Failed to send form-up for fleet {} ({}): {}",
                        fleet.id,
                        fleet.name,
                        e
                    );
                }
            } else {
                tracing::debug!(
                    "Skipping form-up for old fleet {} ({}) from {}",
                    fleet.id,
                    fleet.name,
                    fleet.fleet_time
                );
            }
        }
    }

    Ok(())
}

/// Processes upcoming fleets lists for all configured channels.
///
/// Queries all unique Discord channels that have fleet categories configured,
/// then updates or posts the upcoming fleets list message in each channel.
/// This provides users with a consolidated view of upcoming fleets in each
/// configured notification channel.
///
/// Individual channel update failures are logged but don't prevent updates to
/// other channels.
///
/// # Arguments
/// - `db` - Database connection for querying channel and fleet data
/// - `discord_http` - Discord HTTP client for posting/updating list messages
/// - `app_url` - Application URL for generating fleet detail links
///
/// # Returns
/// - `Ok(())` - All channel lists processed (individual update failures are logged)
/// - `Err(DbErr(_))` - Database query failed
async fn process_upcoming_fleets_lists(
    db: &DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
) -> Result<(), AppError> {
    tracing::trace!("Processing upcoming fleets lists update");

    // Get all unique channels that have fleet categories configured
    let channels = entity::prelude::FleetCategoryChannel::find()
        .all(db)
        .await?;

    // Get unique channel IDs
    let mut channel_ids: Vec<u64> = channels
        .into_iter()
        .filter_map(|c| {
            let id = c.channel_id;
            match id.parse::<u64>() {
                Ok(parsed_id) => Some(parsed_id),
                Err(e) => {
                    tracing::error!("Failed to parse Discord channel_id '{}': {}", id, e);
                    None
                }
            }
        })
        .collect();

    channel_ids.sort();
    channel_ids.dedup();

    tracing::debug!(
        "Updating upcoming fleets lists for {} channels",
        channel_ids.len()
    );

    let notification_service = FleetNotificationService::new(db, discord_http, app_url);

    for channel_id in channel_ids {
        if let Err(e) = notification_service
            .post_upcoming_fleets_list(channel_id)
            .await
        {
            tracing::error!(
                "Failed to update upcoming fleets list for channel {}: {}",
                channel_id,
                e
            );
        } else {
            tracing::debug!(
                "Successfully updated upcoming fleets list for channel {}",
                channel_id
            );
        }
    }

    tracing::trace!("Completed upcoming fleets lists update");

    Ok(())
}
