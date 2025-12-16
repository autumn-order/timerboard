use dioxus::prelude::*;
use std::collections::HashMap;

use crate::{
    client::{
        component::{
            searchable_dropdown::{DropdownItem, SearchableDropdown},
            UtcDateTimeInput,
        },
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
    // Optional props for category selection (only used in create mode)
    #[props(default = None)] selected_category_id: Option<Signal<i32>>,
    #[props(default = None)] manageable_categories: Option<
        Signal<Option<Result<Vec<FleetCategoryListItemDto>, ApiError>>>,
    >,
    // Optional props for datetime validation (only used in edit mode)
    #[props(default = false)] allow_past_time: bool,
    #[props(default = None)] min_datetime: Option<chrono::DateTime<chrono::Utc>>,
) -> Element {
    let mut commander_search = use_signal(|| String::new());
    let mut show_commander_dropdown = use_signal(|| false);

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

                    // Fleet DateTime (UTC, 24-hour format)
                    div {
                        class: "flex flex-col gap-2",
                        label {
                            class: "label",
                            span { class: "label-text", "Fleet Date & Time" }
                        }
                        UtcDateTimeInput {
                            value: fleet_datetime,
                            required: true,
                            disabled: is_submitting,
                            allow_past: allow_past_time,
                            min_datetime: min_datetime,
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
                                                    d: "M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex-1",
                                            div { class: "font-semibold text-sm", "Reminder Ping" }
                                            div { class: "text-sm opacity-80", "{format_duration(&ping_reminder)} before fleet starts" }
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
