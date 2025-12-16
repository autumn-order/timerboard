use dioxus::prelude::*;

use crate::{
    client::component::{DropdownItem, SearchableDropdown},
    model::ping_format::PingFormatDto,
};

use super::{
    duration::validate_duration_input,
    tabs::{AccessRolesTab, ChannelsTab, PingRolesTab},
    types::{ConfigTab, FormFieldsData, ValidationErrorsData},
};

/// Reusable form fields component for fleet category forms
#[component]
pub fn FleetCategoryFormFields(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    validation_errors: Signal<ValidationErrorsData>,
    is_submitting: bool,
    ping_formats: Signal<Vec<PingFormatDto>>,
) -> Element {
    // Create mutable signal for ping format search
    let mut ping_format_search = use_signal(String::new);
    let mut ping_format_dropdown_open = use_signal(|| false);

    // Filter ping formats based on search query
    let filtered_formats = use_memo(move || {
        let formats = ping_formats();
        let query = ping_format_search().to_lowercase();
        if query.is_empty() {
            formats
        } else {
            formats
                .into_iter()
                .filter(|f| f.name.to_lowercase().contains(&query))
                .collect::<Vec<_>>()
        }
    });

    // Get selected format name for display
    let selected_format_name = use_memo(move || {
        let formats = ping_formats();
        if let Some(id) = form_fields().ping_format_id {
            formats.iter().find(|f| f.id == id).map(|f| f.name.clone())
        } else {
            None
        }
    });

    rsx! {
        // Top section - horizontal layout for better space usage
        div {
            class: "grid grid-cols-1 md:grid-cols-2 gap-4",

            // Category Name Input
            div {
                class: "form-control w-full flex flex-col gap-2",
                label {
                    class: "label",
                    span {
                        class: "label-text",
                        "Category Name"
                    }
                }
                input {
                    r#type: "text",
                    class: "input input-bordered w-full",
                    placeholder: "e.g., Structure Timers",
                    value: "{form_fields().category_name}",
                    oninput: move |evt| {
                        form_fields.write().category_name = evt.value();
                    },
                    disabled: is_submitting,
                    required: true,
                }
            }

            // Ping Format Dropdown with Search
            div {
                class: "form-control w-full flex flex-col gap-2",
                label {
                    class: "label",
                    span {
                        class: "label-text",
                        "Ping Format"
                    }
                }
                SearchableDropdown {
                    search_query: ping_format_search,
                    placeholder: "Search ping formats...".to_string(),
                    display_value: selected_format_name(),
                    disabled: is_submitting,
                    required: true,
                    has_items: !filtered_formats().is_empty(),
                    show_dropdown_signal: Some(ping_format_dropdown_open),
                    for format in filtered_formats() {
                        DropdownItem {
                            key: "{format.id}",
                            selected: Some(format.id) == form_fields().ping_format_id,
                            on_select: move |_| {
                                form_fields.write().ping_format_id = Some(format.id);
                                ping_format_search.set(String::new());
                                ping_format_dropdown_open.set(false);
                            },
                            "{format.name}"
                        }
                    }
                }
                label {
                    class: "label",
                    span {
                        class: "label-text-alt",
                        "Select the ping format to use for this fleet category"
                    }
                }
            }
        }

        // Duration fields - horizontal layout
        div {
            class: "grid grid-cols-1 md:grid-cols-3 gap-4",

            // Ping Cooldown Input
            div {
                class: "form-control w-full flex flex-col gap-2",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Ping Cooldown (optional)"
                }
            }
            input {
                r#type: "text",
                class: if validation_errors().ping_cooldown.is_some() { "input input-bordered input-error w-full" } else { "input input-bordered w-full" },
                placeholder: "e.g., 1h, 30m, 1h30m",
                value: "{form_fields().ping_cooldown_str}",
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().ping_cooldown_str = value.clone();
                    validation_errors.write().ping_cooldown = validate_duration_input(&value);
                },
                disabled: is_submitting,
            }
            if let Some(error) = &validation_errors().ping_cooldown {
                div {
                    class: "text-error text-sm mt-1",
                    "{error}"
                }
            }
                label {
                    class: "label",
                    span {
                        class: "label-text-alt text-xs",
                        "Min time between fleets"
                    }
                }
            }

            // Ping Reminder Input
            div {
                class: "form-control w-full flex flex-col gap-2",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Ping Reminder (optional)"
                }
            }
            input {
                r#type: "text",
                class: if validation_errors().ping_reminder.is_some() { "input input-bordered input-error w-full" } else { "input input-bordered w-full" },
                placeholder: "e.g., 15m, 30m",
                value: "{form_fields().ping_reminder_str}",
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().ping_reminder_str = value.clone();
                    validation_errors.write().ping_reminder = validate_duration_input(&value);
                },
                disabled: is_submitting,
            }
            if let Some(error) = &validation_errors().ping_reminder {
                div {
                    class: "text-error text-sm mt-1",
                    "{error}"
                }
            }
                label {
                    class: "label",
                    span {
                        class: "label-text-alt text-xs",
                        "Reminder before fleet"
                    }
                }
            }

            // Max Pre-Ping Input
            div {
                class: "form-control w-full flex flex-col gap-2",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Max Pre-Ping (optional)"
                }
            }
            input {
                r#type: "text",
                class: if validation_errors().max_pre_ping.is_some() { "input input-bordered input-error w-full" } else { "input input-bordered w-full" },
                placeholder: "e.g., 2h, 3h",
                value: "{form_fields().max_pre_ping_str}",
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().max_pre_ping_str = value.clone();
                    validation_errors.write().max_pre_ping = validate_duration_input(&value);
                },
                disabled: is_submitting,
            }
            if let Some(error) = &validation_errors().max_pre_ping {
                div {
                    class: "text-error text-sm mt-1",
                    "{error}"
                }
            }
                label {
                    class: "label",
                    span {
                        class: "label-text-alt text-xs",
                        "Max advance notice"
                    }
                }
            }
        }

        // Divider
        div {
            class: "divider"
        }

        // Tabbed Configuration Section
        ConfigurationTabs {
            guild_id,
            form_fields,
            is_submitting
        }
    }
}

/// Configuration tabs component for roles and channels
#[component]
fn ConfigurationTabs(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
) -> Element {
    let active_tab = form_fields().active_tab;

    rsx! {
        div {
            class: "w-full",
            // Tab buttons
            div {
                class: "tabs tabs-boxed",
                role: "tablist",
                button {
                    r#type: "button",
                    class: if active_tab == ConfigTab::AccessRoles { "tab tab-active" } else { "tab" },
                    onclick: move |_| form_fields.write().active_tab = ConfigTab::AccessRoles,
                    disabled: is_submitting,
                    "Access Roles"
                }
                button {
                    r#type: "button",
                    class: if active_tab == ConfigTab::PingRoles { "tab tab-active" } else { "tab" },
                    onclick: move |_| form_fields.write().active_tab = ConfigTab::PingRoles,
                    disabled: is_submitting,
                    "Ping Roles"
                }
                button {
                    r#type: "button",
                    class: if active_tab == ConfigTab::Channels { "tab tab-active" } else { "tab" },
                    onclick: move |_| form_fields.write().active_tab = ConfigTab::Channels,
                    disabled: is_submitting,
                    "Channels"
                }
            }

            // Tab content
            div {
                class: "mt-4",
                match active_tab {
                    ConfigTab::AccessRoles => rsx! {
                        AccessRolesTab {
                            guild_id,
                            form_fields,
                            is_submitting
                        }
                    },
                    ConfigTab::PingRoles => rsx! {
                        PingRolesTab {
                            guild_id,
                            form_fields,
                            is_submitting
                        }
                    },
                    ConfigTab::Channels => rsx! {
                        ChannelsTab {
                            guild_id,
                            form_fields,
                            is_submitting
                        }
                    }
                }
            }
        }
    }
}
