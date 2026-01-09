use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;

use crate::model::{
    category::{FleetCategoryListItemDto, PingFormatFieldDto},
    discord::DiscordGuildMemberDto,
    ping_format::PingFormatFieldType,
};

use super::{
    format_duration,
    inputs::{
        CategorySelectDropdown, FleetCommanderDropdown, FleetDateTimeInput, FleetNameInput,
        FleetTextField, FleetToggleField, ToggleHiddenFleet, TogglePingReminder,
    },
    CategoryRules,
};

#[component]
pub(super) fn FleetRequiredFields(
    fleet_name: Signal<String>,
    // Category
    selected_category_id: Option<Signal<i32>>,
    field_values: Signal<HashMap<i32, String>>,
    category_name: String,
    manageable_categories: Vec<FleetCategoryListItemDto>,
    // Fleet Time
    fleet_datetime: Signal<String>,
    datetime_error: Signal<Option<String>>,
    allow_past_time: bool,
    min_datetime: Option<DateTime<Utc>>,
    // Fleet Commander
    guild_members: Vec<DiscordGuildMemberDto>,
    fleet_commander_id: Signal<Option<u64>>,
    current_user_id: Option<u64>,
    // Shared
    is_submitting: bool,
) -> Element {
    rsx!(
        div {
            class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            CategorySelectDropdown {
                selected_category_id,
                field_values,
                category_name,
                manageable_categories,
                is_submitting,
            }
            FleetNameInput {
                fleet_name,
                is_submitting,
            }
            FleetDateTimeInput {
                fleet_datetime,
                datetime_error,
                allow_past_time,
                min_datetime,
                is_submitting,
            }
            FleetCommanderDropdown {
                guild_members,
                fleet_commander_id,
                current_user_id,
                is_submitting,
            }
        }
    )
}

#[component]
pub(super) fn FleetVisibilityOptions(
    ping_reminder: Option<Duration>,
    disable_reminder: Signal<bool>,
    is_hidden_fleet: Signal<bool>,
    is_submitting: bool,
) -> Element {
    rsx!(
        div {
            class: "space-y-2 w-full",
            h3 {
                class: "text-lg font-bold",
                "Visibility Options"
            }
            div {
                class: "space-y-3 w-full",
                ToggleHiddenFleet {
                    is_hidden_fleet,
                    is_submitting,
                }

                if ping_reminder.is_some() {
                    TogglePingReminder {
                        disable_reminder,
                        is_submitting
                    }
                }
            }
        }
    )
}

#[component]
pub(super) fn PingFormatCustomFields(
    fields: Vec<PingFormatFieldDto>,
    field_values: Signal<HashMap<i32, String>>,
    is_submitting: bool,
) -> Element {
    rsx!(
        div {
            class: "space-y-4",

            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                for field in fields {
                    {
                        let field_id = field.id;
                        let field_name = field.name.clone();
                        let field_type = field.field_type.clone();

                        rsx! {
                            div {
                                key: "{field_id}",
                                class: "flex flex-col gap-2",
                                label {
                                    class: "label",
                                    span { class: "label-text", "{field_name}" }
                                }

                                match field_type {
                                    PingFormatFieldType::Bool => rsx! {
                                        FleetToggleField {
                                            field,
                                            field_values,
                                            is_submitting
                                        }
                                    },
                                    PingFormatFieldType::Text => rsx! {
                                        FleetTextField {
                                            field,
                                            field_values,
                                            is_submitting
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
pub(super) fn FleetCategoryRules(
    category_name: String,
    category_rules: CategoryRules,
    disable_reminder: bool,
) -> Element {
    rsx!(
        div {
            class: "divider mt-6 mb-4"
        }
        div {
            class: "space-y-3",
            h3 {
                class: "text-lg font-bold mb-3",
                "{category_name} Rules"
            }
            div {
                class: "card bg-base-200 shadow-sm",
                div {
                    class: "card-body p-4 space-y-3",
                    if let Some(ping_group_name) = category_rules.ping_group_name {
                        div {
                            class: "flex items-center gap-3",
                            div {
                                class: "badge badge-primary badge-lg gap-2 h-auto py-2",
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
                                        d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                                    }
                                }
                            }
                            div {
                                class: "flex-1",
                                div { class: "font-semibold text-sm", "Ping Group" }
                                if let Some(cooldown) = category_rules.ping_group_cooldown {
                                    div {
                                        class: "text-sm opacity-80",
                                        "{ping_group_name} (shared {format_duration(&cooldown)} cooldown)"
                                    }
                                } else {
                                    div {
                                        class: "text-sm opacity-80",
                                        "{ping_group_name}"
                                    }
                                }
                            }
                        }
                    }
                    if let Some(ping_lead_time) = category_rules.ping_lead_time {
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
                    if let Some(max_pre_ping) = category_rules.max_pre_ping {
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
                    if let Some(ping_reminder) = category_rules.ping_reminder {
                        div {
                            class: "flex items-center gap-3",
                            div {
                                class: if disable_reminder {
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
                                    class: if disable_reminder {
                                        "font-semibold text-sm line-through opacity-50"
                                    } else {
                                        "font-semibold text-sm"
                                    },
                                    "Reminder Ping"
                                }
                                div {
                                    class: if disable_reminder {
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
    )
}
