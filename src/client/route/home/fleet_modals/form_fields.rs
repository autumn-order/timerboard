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
