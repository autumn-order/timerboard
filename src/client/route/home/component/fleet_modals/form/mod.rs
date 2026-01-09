mod inputs;
mod layout;

use std::collections::HashMap;

use chrono::TimeDelta;
use dioxus::prelude::*;

use crate::{
    client::model::error::ApiError,
    model::{
        category::{FleetCategoryDetailsDto, FleetCategoryListItemDto},
        discord::DiscordGuildMemberDto,
    },
};

use self::{
    inputs::FleetDescription,
    layout::{
        FleetCategoryRules, FleetRequiredFields, FleetVisibilityOptions, PingFormatCustomFields,
    },
};

/// Helper function to format a duration for display
fn format_duration(duration: &chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 && minutes > 0 {
        format!(
            "{} hour{} {} minute{}",
            hours,
            if hours == 1 { "" } else { "s" },
            minutes,
            if minutes == 1 { "" } else { "s" }
        )
    } else if hours > 0 {
        format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else {
        format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" })
    }
}

#[derive(Clone, PartialEq)]
struct CategoryRules {
    ping_group_name: Option<String>,
    ping_group_cooldown: Option<TimeDelta>,
    ping_lead_time: Option<TimeDelta>,
    max_pre_ping: Option<TimeDelta>,
    ping_reminder: Option<TimeDelta>,
}

/// Shared form fields component for fleet creation and editing
#[component]
pub fn FleetFormFields(
    guild_id: u64,
    mut fleet_name: Signal<String>,
    mut fleet_datetime: Signal<String>,
    mut fleet_commander_id: Signal<Option<u64>>,
    mut fleet_description: Signal<String>,
    mut field_values: Signal<HashMap<i32, String>>,
    category_details: Signal<Option<Result<FleetCategoryDetailsDto, ApiError>>>,
    guild_members: Signal<Vec<DiscordGuildMemberDto>>,
    is_submitting: bool,
    current_user_id: Option<u64>,
    // Fleet visibility options
    mut hidden: Signal<bool>,
    mut disable_reminder: Signal<bool>,
    // Optional props for category selection (only used in create mode)
    #[props(default = None)] selected_category_id: Option<Signal<i32>>,
    manageable_categories: Signal<Vec<FleetCategoryListItemDto>>,
    // Optional props for datetime validation (only used in edit mode)
    #[props(default = false)] allow_past_time: bool,
    #[props(default = None)] min_datetime: Option<chrono::DateTime<chrono::Utc>>,
    // Optional signal to expose datetime validation errors to parent
    #[props(default = None)] datetime_error_signal: Option<Signal<Option<String>>>,
) -> Element {
    // Use provided signal or create local one
    let local_datetime_error = use_signal(|| None::<String>);
    let datetime_error = datetime_error_signal.unwrap_or(local_datetime_error);

    rsx! {
        if let Some(Ok(details)) = category_details() {
            div {
                class: "space-y-4",

                FleetRequiredFields {
                    fleet_name,
                    // Fleet Category
                    selected_category_id,
                    field_values,
                    category_name: details.name.to_string(),
                    manageable_categories: manageable_categories(),
                    // Fleet Time
                    fleet_datetime,
                    datetime_error,
                    allow_past_time,
                    min_datetime,
                    // Fleet Commander
                    guild_members: guild_members(),
                    fleet_commander_id,
                    current_user_id,
                    // Shared
                    is_submitting,
                }

                if !details.fields.is_empty() {
                    PingFormatCustomFields {
                        fields: details.fields,
                        field_values,
                        is_submitting,
                    }
                }

                FleetDescription {
                    fleet_description,
                    is_submitting,
                }
                FleetVisibilityOptions {
                    ping_reminder: details.ping_reminder,
                    disable_reminder: disable_reminder,
                    is_hidden_fleet: hidden,
                    is_submitting,
                }

                // TODO: `details.category_rules.is_some()` instead
                if details.ping_lead_time.is_some() || details.ping_reminder.is_some() || details.max_pre_ping.is_some() || details.ping_group_name.is_some() {
                    FleetCategoryRules {
                        category_name: details.name,
                        category_rules: CategoryRules {
                            ping_group_name: details.ping_group_name,
                            ping_group_cooldown: details.ping_group_cooldown,
                            ping_lead_time: details.ping_lead_time,
                            max_pre_ping: details.max_pre_ping,
                            ping_reminder: details.ping_reminder,
                        },
                        disable_reminder: disable_reminder()
                    }
                }
            }
        } else if let Some(Err(_)) = category_details() {
            div {
                class: "text-center py-8",
                p {
                    class: "text-error",
                    "Failed to load category details"
                }
            }
        } else {
            div {
                class: "text-center py-8",
                span {
                    class: "loading loading-spinner loading-lg"
                }
            }
        }
    }
}
