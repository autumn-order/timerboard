use chrono::Utc;
use dioxus_logger::tracing;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::http::Http;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::server::{error::AppError, service::fleet_notification::FleetNotificationService};

/// Starts the fleet notification scheduler
///
/// This scheduler runs every minute and checks for:
/// - Fleets needing reminder notifications (based on category's ping_reminder time)
/// - Fleets needing form-up notifications (fleet_time has passed)
///
/// # Arguments
/// - `db`: Database connection
/// - `discord_http`: Discord HTTP client for sending notifications
/// - `app_url`: Application URL for embed links
pub async fn start_scheduler(
    db: DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
) -> Result<(), AppError> {
    let scheduler = JobScheduler::new().await?;

    // Clone resources for the job
    let job_db = db.clone();
    let job_http = discord_http.clone();
    let job_app_url = app_url.clone();

    // Schedule job to run every minute
    let job = Job::new_async("0 * * * * *", move |_uuid, _lock| {
        let db = job_db.clone();
        let http = job_http.clone();
        let app_url = job_app_url.clone();

        Box::pin(async move {
            if let Err(e) = process_fleet_notifications(&db, http, app_url).await {
                tracing::error!("Error processing fleet notifications: {}", e);
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    tracing::info!("Fleet notification scheduler started");

    Ok(())
}

/// Processes fleet notifications for reminders and form-ups
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

/// Processes fleets needing reminder notifications
async fn process_reminders(
    db: &DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
    now: chrono::DateTime<Utc>,
) -> Result<(), AppError> {
    // Query fleets that need reminders:
    // 1. Not hidden
    // 2. Reminders not disabled
    // 3. Haven't been reminded yet (no reminder message)
    // 4. Reminder time has passed (fleet_time - category.ping_reminder <= now)

    let fleets = entity::prelude::Fleet::find()
        .filter(entity::fleet::Column::Hidden.eq(false))
        .filter(entity::fleet::Column::DisableReminder.eq(false))
        .all(db)
        .await?;

    for fleet in fleets {
        // Get category to check ping_reminder
        let category = entity::prelude::FleetCategory::find_by_id(fleet.category_id)
            .one(db)
            .await?;

        if let Some(category) = category {
            if let Some(reminder_seconds) = category.ping_reminder {
                // Calculate reminder time
                let reminder_duration = chrono::Duration::seconds(reminder_seconds as i64);
                let reminder_time = fleet.fleet_time - reminder_duration;

                // Check if reminder time has passed
                if now >= reminder_time && now < fleet.fleet_time {
                    // Check if reminder already sent
                    let existing_reminder = entity::prelude::FleetMessage::find()
                        .filter(entity::fleet_message::Column::FleetId.eq(fleet.id))
                        .filter(entity::fleet_message::Column::MessageType.eq("reminder"))
                        .one(db)
                        .await?;

                    if existing_reminder.is_none() {
                        // Send reminder
                        tracing::info!("Sending reminder for fleet {} ({})", fleet.id, fleet.name);

                        let notification_service = FleetNotificationService::new(
                            db,
                            discord_http.clone(),
                            app_url.clone(),
                        );

                        // Get field values
                        let field_values = entity::prelude::FleetFieldValue::find()
                            .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
                            .all(db)
                            .await?;

                        let field_values_map: std::collections::HashMap<i32, String> = field_values
                            .into_iter()
                            .map(|fv| (fv.field_id, fv.value))
                            .collect();

                        if let Err(e) = notification_service
                            .post_fleet_reminder(&fleet, &field_values_map)
                            .await
                        {
                            tracing::error!(
                                "Failed to send reminder for fleet {}: {}",
                                fleet.id,
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

/// Processes fleets needing form-up notifications
async fn process_formups(
    db: &DatabaseConnection,
    discord_http: Arc<Http>,
    app_url: String,
    now: chrono::DateTime<Utc>,
) -> Result<(), AppError> {
    // Query fleets that need form-up notifications:
    // 1. Fleet time has passed
    // 2. Haven't sent form-up yet (no formup message)

    let fleets = entity::prelude::Fleet::find()
        .filter(entity::fleet::Column::FleetTime.lte(now))
        .all(db)
        .await?;

    for fleet in fleets {
        // Check if form-up already sent
        let existing_formup = entity::prelude::FleetMessage::find()
            .filter(entity::fleet_message::Column::FleetId.eq(fleet.id))
            .filter(entity::fleet_message::Column::MessageType.eq("formup"))
            .one(db)
            .await?;

        if existing_formup.is_none() {
            // Only send form-up if fleet time is within the last 5 minutes
            // This prevents sending form-ups for very old fleets
            let five_minutes_ago = now - chrono::Duration::minutes(5);

            if fleet.fleet_time >= five_minutes_ago {
                tracing::info!("Sending form-up for fleet {} ({})", fleet.id, fleet.name);

                let notification_service =
                    FleetNotificationService::new(db, discord_http.clone(), app_url.clone());

                // Get field values
                let field_values = entity::prelude::FleetFieldValue::find()
                    .filter(entity::fleet_field_value::Column::FleetId.eq(fleet.id))
                    .all(db)
                    .await?;

                let field_values_map: std::collections::HashMap<i32, String> = field_values
                    .into_iter()
                    .map(|fv| (fv.field_id, fv.value))
                    .collect();

                if let Err(e) = notification_service
                    .post_fleet_formup(&fleet, &field_values_map)
                    .await
                {
                    tracing::error!("Failed to send form-up for fleet {}: {}", fleet.id, e);
                }
            }
        }
    }

    Ok(())
}
