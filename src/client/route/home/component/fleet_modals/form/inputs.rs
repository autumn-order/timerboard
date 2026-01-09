use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;

use crate::{
    client::component::searchable_dropdown::{DropdownItem, SearchableDropdown},
    model::{
        category::{FleetCategoryListItemDto, PingFormatFieldDto},
        discord::DiscordGuildMemberDto,
    },
};

#[component]
pub(super) fn CategorySelectDropdown(
    selected_category_id: Option<Signal<i32>>,
    field_values: Signal<HashMap<i32, String>>,
    category_name: String,
    manageable_categories: Vec<FleetCategoryListItemDto>,
    is_submitting: bool,
) -> Element {
    rsx!(
        // Category (dropdown to switch categories if in create mode)
        if let (Some(mut category_id), categories) = (selected_category_id, manageable_categories) {
            div {
                class: "flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text", "Category" }
                }
                select {
                    class: "select select-bordered w-full",
                    value: "{category_id()}",
                    disabled: is_submitting,
                    onchange: move |e| {
                        if let Ok(new_id) = e.value().parse::<i32>() {
                            category_id.set(new_id);
                            // Clear field values when category changes
                            field_values.set(HashMap::new());
                        }
                    },
                    for category in categories {
                        option {
                            key: "{category.id}",
                            value: "{category.id}",
                            selected: category.id == category_id(),
                            "{category.name}"
                        }
                    }
                }
            }
        }
    )
}

#[component]
pub(super) fn FleetNameInput(mut fleet_name: Signal<String>, is_submitting: bool) -> Element {
    rsx!(
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
    )
}

// Fleet DateTime (Local time input with UTC conversion display)
#[component]
pub(super) fn FleetDateTimeInput(
    fleet_datetime: Signal<String>,
    datetime_error: Signal<Option<String>>,
    allow_past_time: bool,
    min_datetime: Option<DateTime<Utc>>,
    is_submitting: bool,
) -> Element {
    rsx!(
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
    )
}

// Fleet Commander - Searchable Dropdown
#[component]
pub(super) fn FleetCommanderDropdown(
    guild_members: Vec<DiscordGuildMemberDto>,
    fleet_commander_id: Signal<Option<u64>>,
    current_user_id: Option<u64>,
    is_submitting: bool,
) -> Element {
    let mut commander_search = use_signal(String::new);
    let mut show_commander_dropdown = use_signal(|| false);

    rsx!(
        div {
            class: "flex flex-col gap-2",
            label {
                class: "label",
                span { class: "label-text", "Fleet Commander" }
            }
            {
                let selected_member = guild_members.iter().find(|m| Some(m.user_id) == fleet_commander_id());
                let display_value = selected_member.map(|m| format!("{} (@{})", m.display_name, m.username));

                let search_lower = commander_search().to_lowercase();
                let mut filtered_members: Vec<_> = guild_members.iter()
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
        }
    )
}

#[component]
pub(super) fn FleetDescription(fleet_description: Signal<String>, is_submitting: bool) -> Element {
    rsx!(
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
    )
}

#[component]
pub(super) fn FleetToggleField(
    field: PingFormatFieldDto,
    field_values: Signal<HashMap<i32, String>>,
    is_submitting: bool,
) -> Element {
    rsx!(
        div {
            class: "form-control",
            label {
                class: "label cursor-pointer justify-start gap-3",
                input {
                    r#type: "checkbox",
                    class: "checkbox checkbox-primary",
                    checked: field_values().get(&field.id).map(|v| v == "true").unwrap_or(false),
                    disabled: is_submitting,
                    onchange: move |e| {
                        let mut values = field_values();
                        values.insert(field.id, if e.checked() { "true".to_string() } else { "false".to_string() });
                        field_values.set(values);
                    }
                }
                span { class: "label-text", "Enable {field.name}" }
            }
        }
    )
}

#[component]
pub(super) fn FleetTextField(
    field: PingFormatFieldDto,
    field_values: Signal<HashMap<i32, String>>,
    is_submitting: bool,
) -> Element {
    let default_values = field.default_field_values.clone();

    let current_value = field_values().get(&field.id).cloned().unwrap_or_default();
    let has_defaults = !default_values.is_empty();

    rsx! {
        div {
            class: "relative",
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: if has_defaults {
                    format!("Enter or select {}", field.name.to_lowercase())
                } else {
                    format!("Enter {}", field.name.to_lowercase())
                },
                value: "{current_value}",
                disabled: is_submitting,
                list: if has_defaults { format!("datalist-{}", field.id) } else { String::new() },
                oninput: move |e| {
                    let mut values = field_values();
                    values.insert(field.id, e.value());
                    field_values.set(values);
                }
            }

            // HTML5 datalist for autocomplete
            if has_defaults {
                datalist {
                    id: "datalist-{field.id}",
                    for value in &default_values {
                        option {
                            value: "{value}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn ToggleHiddenFleet(is_hidden_fleet: Signal<bool>, is_submitting: bool) -> Element {
    rsx!(
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
                    checked: is_hidden_fleet(),
                    disabled: is_submitting,
                    onchange: move |e| is_hidden_fleet.set(e.checked())
                }
            }
        }
    )
}

#[component]
pub(super) fn TogglePingReminder(disable_reminder: Signal<bool>, is_submitting: bool) -> Element {
    rsx!(
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
    )
}
