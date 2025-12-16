use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;

use crate::{
    client::{
        component::searchable_dropdown::{DropdownItem, SearchableDropdown},
        model::error::ApiError,
    },
    model::{
        category::FleetCategoryDetailsDto, category::FleetCategoryListItemDto,
        discord::DiscordGuildMemberDto,
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
    guild_members: Signal<Option<Result<Vec<DiscordGuildMemberDto>, ApiError>>>,
    is_submitting: bool,
    current_user_id: Option<u64>,
    // Fleet visibility options
    mut hidden: Signal<bool>,
    mut disable_reminder: Signal<bool>,
    // Optional props for category selection (only used in create mode)
    #[props(default = None)] selected_category_id: Option<Signal<i32>>,
    #[props(default = None)] manageable_categories: Option<
        Signal<Option<Result<Vec<FleetCategoryListItemDto>, ApiError>>>,
    >,
    // Optional props for datetime validation (only used in edit mode)
    #[props(default = false)] allow_past_time: bool,
    #[props(default = None)] min_datetime: Option<chrono::DateTime<chrono::Utc>>,
    // Optional signal to expose datetime validation errors to parent
    #[props(default = None)] datetime_error_signal: Option<Signal<Option<String>>>,
) -> Element {
    let mut commander_search = use_signal(|| String::new());
    let mut show_commander_dropdown = use_signal(|| false);

    // Use provided signal or create local one
    let local_datetime_error = use_signal(|| None::<String>);
    let mut datetime_error = datetime_error_signal.unwrap_or(local_datetime_error);

    rsx! {
        if let Some(Ok(details)) = category_details() {
            div {
                class: "space-y-4",

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    // Category (dropdown to switch categories if in create mode)
                    if let (Some(mut cat_id), Some(cats)) = (selected_category_id, manageable_categories) {
                        div {
                            class: "flex flex-col gap-2",
                            label {
                                class: "label",
                                span { class: "label-text", "Category" }
                            }
                            {
                                if let Some(Ok(categories)) = cats() {
                                    rsx! {
                                        select {
                                            class: "select select-bordered w-full",
                                            value: "{cat_id()}",
                                            disabled: is_submitting,
                                            onchange: move |e| {
                                                if let Ok(new_id) = e.value().parse::<i32>() {
                                                    cat_id.set(new_id);
                                                    // Clear field values when category changes
                                                    field_values.set(HashMap::new());
                                                }
                                            },
                                            for category in categories {
                                                option {
                                                    key: "{category.id}",
                                                    value: "{category.id}",
                                                    selected: category.id == cat_id(),
                                                    "{category.name}"
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        input {
                                            r#type: "text",
                                            class: "input input-bordered w-full",
                                            value: "{details.name}",
                                            disabled: true
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Fleet Name
                    div {
                        class: "flex flex-col gap-2",
                        label {
                            class: "label",
                            span { class: "label-text", "Fleet Name" }
                        }
                        input {
                            r#type: "text",
                            class: "input input-bordered w-full",
                            placeholder: "Enter fleet name...",
                            value: "{fleet_name}",
                            disabled: is_submitting,
                            required: true,
                            oninput: move |e| fleet_name.set(e.value())
                        }
                    }

                    // Fleet DateTime (Local time input with UTC conversion display)
                    div {
                        class: "flex flex-col gap-2",
                        label {
                            class: "label",
                            span { class: "label-text", "Fleet Date & Time (Local)" }
                        }
                        {
                            // Convert stored UTC string to local datetime-local format
                            let local_value = if !fleet_datetime().is_empty() {
                                if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&fleet_datetime(), "%Y-%m-%d %H:%M") {
                                    let utc_dt = Utc.from_utc_datetime(&naive_dt);
                                    let local_dt: DateTime<Local> = utc_dt.into();
                                    // Format as datetime-local: YYYY-MM-DDTHH:MM
                                    local_dt.format("%Y-%m-%dT%H:%M").to_string()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            // Calculate min attribute for validation
                            let min_attr = if !allow_past_time {
                                if let Some(min_dt) = min_datetime {
                                    // Use min_datetime if provided (edit mode)
                                    let local_min: DateTime<Local> = min_dt.into();
                                    Some(local_min.format("%Y-%m-%dT%H:%M").to_string())
                                } else {
                                    // Use current time (create mode)
                                    let now_local = Local::now();
                                    Some(now_local.format("%Y-%m-%dT%H:%M").to_string())
                                }
                            } else {
                                None
                            };

                            rsx! {
                                input {
                                    r#type: "datetime-local",
                                    class: if datetime_error().is_some() {
                                        "input input-bordered input-error w-full"
                                    } else {
                                        "input input-bordered w-full"
                                    },
                                    value: "{local_value}",
                                    min: min_attr.as_deref(),
                                    disabled: is_submitting,
                                    required: true,
                                    oninput: move |e| {
                                        let local_input = e.value();
                                        datetime_error.set(None);

                                        if !local_input.is_empty() {
                                            // Parse local datetime-local format: YYYY-MM-DDTHH:MM
                                            if let Ok(naive_local) = NaiveDateTime::parse_from_str(&local_input, "%Y-%m-%dT%H:%M") {
                                                // Assume input is in local timezone, convert to UTC
                                                let local_dt = Local.from_local_datetime(&naive_local).single();
                                                if let Some(local_dt) = local_dt {
                                                    let utc_dt: DateTime<Utc> = local_dt.into();

                                                    // Always store the value first
                                                    fleet_datetime.set(utc_dt.format("%Y-%m-%d %H:%M").to_string());

                                                    // Then validate against min_datetime if provided
                                                    if let Some(min_dt) = min_datetime {
                                                        if utc_dt < min_dt {
                                                            datetime_error.set(Some(format!(
                                                                "Fleet time cannot be earlier than the original time ({})",
                                                                min_dt.format("%Y-%m-%d %H:%M UTC")
                                                            )));
                                                            return;
                                                        }
                                                    }

                                                    // Validate against current time if not allowing past times
                                                    // Allow a 2-minute grace period for immediate fleets to handle:
                                                    // - Time spent filling out the form
                                                    // - Clock skew between client and server
                                                    if !allow_past_time {
                                                        let now = Utc::now();
                                                        let grace_period = chrono::Duration::minutes(2);
                                                        let min_allowed_time = now - grace_period;

                                                        if utc_dt < min_allowed_time {
                                                            datetime_error.set(Some("Fleet time cannot be more than 2 minutes in the past".to_string()));
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            fleet_datetime.set(String::new());
                                        }
                                    }
                                }
                            }
                        }
                        // Display validation error
                        if let Some(error) = datetime_error() {
                            div {
                                class: "text-xs text-error mt-1",
                                "{error}"
                            }
                        }
                        // Display UTC conversion
                        if !fleet_datetime().is_empty() {
                            {
                                let utc_display = if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&fleet_datetime(), "%Y-%m-%d %H:%M") {
                                    let utc_dt = Utc.from_utc_datetime(&naive_dt);
                                    format!("EVE Time: {}", utc_dt.format("%Y-%m-%d %H:%M"))
                                } else {
                                    "Invalid datetime".to_string()
                                };

                                rsx! {
                                    div {
                                        class: "text-xs opacity-70 mt-1",
                                        span { class: "font-mono", "{utc_display}" }
                                    }
                                }
                            }
                        }
                    }

                    // Fleet Commander - Searchable Dropdown
                    div {
                        class: "flex flex-col gap-2",
                        label {
                            class: "label",
                            span { class: "label-text", "Fleet Commander" }
                        }

                        if let Some(Ok(members)) = guild_members() {
                            {
                                let selected_member = members.iter().find(|m| Some(m.user_id) == fleet_commander_id());
                                let display_value = selected_member.map(|m| format!("{} (@{})", m.display_name, m.username));

                                let search_lower = commander_search().to_lowercase();
                                let mut filtered_members: Vec<_> = members.iter()
                                    .filter(|m| {
                                        search_lower.is_empty() ||
                                        m.display_name.to_lowercase().contains(&search_lower) ||
                                        m.username.to_lowercase().contains(&search_lower)
                                    })
                                    .collect();

                                // Sort to always put the logged-in user at the top
                                if let Some(current_id) = current_user_id {
                                    filtered_members.sort_by_key(|m| {
                                        if m.user_id == current_id {
                                            0 // Current user first
                                        } else {
                                            1 // Everyone else after
                                        }
                                    });
                                }

                                rsx! {
                                    SearchableDropdown {
                                        search_query: commander_search,
                                        placeholder: "Search for a fleet commander...".to_string(),
                                        display_value,
                                        required: true,
                                        disabled: is_submitting,
                                        has_items: !filtered_members.is_empty(),
                                        show_dropdown_signal: Some(show_commander_dropdown),
                                        empty_message: "No guild members found".to_string(),
                                        not_found_message: "No matching members found".to_string(),

                                        for member in filtered_members {
                                            {
                                                let member_id = member.user_id;
                                                let member_display = member.display_name.clone();
                                                let member_username = member.username.clone();
                                                let is_selected = Some(member_id) == fleet_commander_id();

                                                rsx! {
                                                    DropdownItem {
                                                        key: "{member_id}",
                                                        selected: is_selected,
                                                        on_select: move |_| {
                                                            fleet_commander_id.set(Some(member_id));
                                                            show_commander_dropdown.set(false);
                                                            commander_search.set(String::new());
                                                        },
                                                        div {
                                                            class: "flex flex-col",
                                                            span { class: "font-semibold", "{member_display}" }
                                                            span { class: "text-sm opacity-70", "@{member_username}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                class: "skeleton h-12 w-full"
                            }
                        }
                    }
                }

                // Ping Format Fields Section
                if !details.fields.is_empty() {
                    div {
                        class: "space-y-4",

                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                            for field in details.fields.clone() {
                                {
                                    let field_id = field.id;
                                    let field_name = field.name.clone();
                                    rsx! {
                                        div {
                                            key: "{field_id}",
                                            class: "flex flex-col gap-2",
                                            label {
                                                class: "label",
                                                span { class: "label-text", "{field_name}" }
                                            }
                                            input {
                                                r#type: "text",
                                                class: "input input-bordered w-full",
                                                placeholder: "Enter {field_name.to_lowercase()}...",
                                                value: "{field_values().get(&field_id).cloned().unwrap_or_default()}",
                                                disabled: is_submitting,
                                                oninput: move |e| {
                                                    let mut values = field_values();
                                                    values.insert(field_id, e.value());
                                                    field_values.set(values);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Fleet Description Section
                div {
                    class: "space-y-2",
                    h3 {
                        class: "text-lg font-bold",
                        "Description"
                    }
                    textarea {
                        class: "textarea textarea-bordered h-32 w-full",
                        placeholder: "Enter fleet description...",
                        value: "{fleet_description}",
                        disabled: is_submitting,
                        oninput: move |e| fleet_description.set(e.value())
                    }
                }

                // Fleet Visibility Options
                div {
                    class: "space-y-2 w-full",
                    h3 {
                        class: "text-lg font-bold",
                        "Visibility Options"
                    }
                    div {
                        class: "space-y-3 w-full",

                        // Hidden toggle
                        div {
                            class: "form-control w-full",
                            label {
                                class: "label cursor-pointer p-3 bg-base-200 rounded-box w-full",
                                div {
                                    class: "flex-1 select-none",
                                    div {
                                        class: "label-text font-semibold",
                                        "Hidden Fleet"
                                    }
                                    div {
                                        class: "label-text-alt text-sm opacity-70",
                                        "Only visible to FCs and category managers until reminder time (or form-up if no reminder)"
                                    }
                                }
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: hidden(),
                                    disabled: is_submitting,
                                    onchange: move |e| hidden.set(e.checked())
                                }
                            }
                        }

                        // Disable reminder toggle (only show if category has a reminder configured)
                        if details.ping_reminder.is_some() {
                            div {
                                class: "form-control w-full",
                                label {
                                    class: "label cursor-pointer p-3 bg-base-200 rounded-box w-full",
                                    div {
                                        class: "flex-1 select-none",
                                        div {
                                            class: "label-text font-semibold",
                                            "Disable Reminder Ping"
                                        }
                                        div {
                                            class: "label-text-alt text-sm opacity-70",
                                            "Skip automated reminder ping for this fleet"
                                        }
                                    }
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-primary",
                                        checked: disable_reminder(),
                                        disabled: is_submitting,
                                        onchange: move |e| disable_reminder.set(e.checked())
                                    }
                                }
                            }
                        }
                    }
                }

                // Category Rules Section
                if details.ping_lead_time.is_some() || details.ping_reminder.is_some() || details.max_pre_ping.is_some() {
                    div {
                        class: "divider mt-6 mb-4"
                    }
                    div {
                        class: "space-y-3",
                        h3 {
                            class: "text-lg font-bold mb-3",
                            "{details.name} Rules"
                        }
                        div {
                            class: "card bg-base-200 shadow-sm",
                            div {
                                class: "card-body p-4 space-y-3",
                                if let Some(ping_lead_time) = details.ping_lead_time {
                                    div {
                                        class: "flex items-center gap-3",
                                        div {
                                            class: "badge badge-neutral badge-lg gap-2 h-auto py-2",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                class: "h-4 w-4",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                stroke: "currentColor",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    stroke_width: "2",
                                                    d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex-1",
                                            div { class: "font-semibold text-sm", "Minimum Gap Between Fleets" }
                                            div { class: "text-sm opacity-80", "{format_duration(&ping_lead_time)}" }
                                        }
                                    }
                                }
                                if let Some(max_pre_ping) = details.max_pre_ping {
                                    div {
                                        class: "flex items-center gap-3",
                                        div {
                                            class: "badge badge-neutral badge-lg gap-2 h-auto py-2",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                class: "h-4 w-4",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                stroke: "currentColor",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    stroke_width: "2",
                                                    d: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex-1",
                                            div { class: "font-semibold text-sm", "Maximum Advance Scheduling" }
                                            div { class: "text-sm opacity-80", "{format_duration(&max_pre_ping)}" }
                                        }
                                    }
                                }
                                if let Some(ping_reminder) = details.ping_reminder {
                                    div {
                                        class: "flex items-center gap-3",
                                        div {
                                            class: if disable_reminder() {
                                                "badge badge-neutral badge-lg gap-2 h-auto py-2 opacity-50"
                                            } else {
                                                "badge badge-neutral badge-lg gap-2 h-auto py-2"
                                            },
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                class: "h-4 w-4",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                stroke: "currentColor",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    stroke_width: "2",
                                                    d: "M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex-1",
                                            div {
                                                class: if disable_reminder() {
                                                    "font-semibold text-sm line-through opacity-50"
                                                } else {
                                                    "font-semibold text-sm"
                                                },
                                                "Reminder Ping"
                                            }
                                            div {
                                                class: if disable_reminder() {
                                                    "text-sm opacity-50 line-through"
                                                } else {
                                                    "text-sm opacity-80"
                                                },
                                                "{format_duration(&ping_reminder)} before fleet starts"
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
