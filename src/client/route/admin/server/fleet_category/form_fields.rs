use dioxus::prelude::*;

use crate::model::{
    discord::{DiscordGuildChannelDto, DiscordGuildRoleDto},
    ping_format::PingFormatDto,
};

use super::duration::validate_duration_input;

#[cfg(feature = "web")]
use crate::client::api::discord::{get_discord_guild_channels, get_discord_guild_roles};

/// Tab selection for the bottom section
#[derive(Clone, Copy, PartialEq)]
pub enum ConfigTab {
    AccessRoles,
    PingRoles,
    Channels,
}

impl Default for ConfigTab {
    fn default() -> Self {
        ConfigTab::AccessRoles
    }
}

/// Role data
#[derive(Clone, PartialEq)]
pub struct RoleData {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Channel data
#[derive(Clone, PartialEq)]
pub struct ChannelData {
    pub id: i64,
    pub name: String,
}

/// Access role with permissions
#[derive(Clone, PartialEq)]
pub struct AccessRoleData {
    pub role: RoleData,
    pub can_view: bool,
    pub can_create: bool,
    pub can_manage: bool,
}

/// Form field values
#[derive(Clone, Default, PartialEq)]
pub struct FormFieldsData {
    pub category_name: String,
    pub ping_format_id: Option<i32>,
    pub search_query: String,
    pub ping_cooldown_str: String,
    pub ping_reminder_str: String,
    pub max_pre_ping_str: String,
    pub active_tab: ConfigTab,
    pub role_search_query: String,
    pub channel_search_query: String,
    pub access_roles: Vec<AccessRoleData>,
    pub ping_roles: Vec<RoleData>,
    pub channels: Vec<ChannelData>,
}

/// Validation errors for duration fields
#[derive(Clone, Default, PartialEq)]
pub struct ValidationErrorsData {
    pub ping_cooldown: Option<String>,
    pub ping_reminder: Option<String>,
    pub max_pre_ping: Option<String>,
}

/// Reusable form fields component for fleet category forms
#[component]
pub fn FleetCategoryFormFields(
    guild_id: u64,
    form_fields: Signal<FormFieldsData>,
    validation_errors: Signal<ValidationErrorsData>,
    is_submitting: bool,
    ping_formats: Signal<Vec<PingFormatDto>>,
) -> Element {
    let mut show_dropdown = use_signal(|| false);

    // Filter ping formats based on search query
    let filtered_formats = use_memo(move || {
        let formats = ping_formats();
        let query = form_fields().search_query.to_lowercase();
        if query.is_empty() {
            formats
        } else {
            formats
                .into_iter()
                .filter(|f| f.name.to_lowercase().contains(&query))
                .collect::<Vec<_>>()
        }
    });

    // Get selected format name
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
            div {
                class: "relative",
                input {
                    r#type: "text",
                    class: "input input-bordered w-full",
                    placeholder: if selected_format_name().is_some() { "" } else { "Search ping formats..." },
                    value: if show_dropdown() {
                        "{form_fields().search_query}"
                    } else if let Some(name) = &selected_format_name() {
                        "{name}"
                    } else {
                        ""
                    },
                    onfocus: move |_| {
                        show_dropdown.set(true);
                        form_fields.write().search_query = String::new();
                    },
                    onblur: move |_| {
                        show_dropdown.set(false);
                    },
                    oninput: move |evt| {
                        form_fields.write().search_query = evt.value();
                        show_dropdown.set(true);
                    },
                    disabled: is_submitting,
                    required: true,
                }
                {
                    let formats = filtered_formats();
                    if show_dropdown() {
                        if !formats.is_empty() {
                            rsx! {
                                div {
                                    class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg max-h-60 overflow-y-auto",
                                    for format in formats {
                                        div {
                                            key: "{format.id}",
                                            class: if Some(format.id) == form_fields().ping_format_id {
                                                "px-4 py-2 cursor-pointer bg-primary text-primary-content hover:bg-primary-focus"
                                            } else {
                                                "px-4 py-2 cursor-pointer hover:bg-base-200"
                                            },
                                            onmousedown: move |evt| {
                                                evt.prevent_default();
                                                form_fields.write().ping_format_id = Some(format.id);
                                                form_fields.write().search_query = String::new();
                                                show_dropdown.set(false);
                                            },
                                            "{format.name}"
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg",
                                    div {
                                        class: "px-4 py-2 text-center opacity-50",
                                        if !form_fields().search_query.is_empty() {
                                            "No ping formats found"
                                        } else {
                                            "No ping formats available"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
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

/// Access Roles tab content
#[component]
fn AccessRolesTab(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let mut available_roles = use_signal(|| Vec::<DiscordGuildRoleDto>::new());

    // Fetch roles when component mounts
    #[cfg(feature = "web")]
    let roles_future =
        use_resource(move || async move { get_discord_guild_roles(guild_id, 0, 1000).await.ok() });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = roles_future.read_unchecked().as_ref() {
            available_roles.set(result.roles.clone());
        }
    });

    // Filter available roles by search query and exclude already added roles
    let filtered_roles = use_memo(move || {
        let roles = available_roles();
        let query = form_fields().role_search_query.to_lowercase();
        let access_role_ids: Vec<i64> = form_fields()
            .access_roles
            .iter()
            .map(|ar| ar.role.id)
            .collect();

        let mut filtered: Vec<_> = roles
            .into_iter()
            .filter(|r| !access_role_ids.contains(&r.role_id))
            .filter(|r| {
                if query.is_empty() {
                    true
                } else {
                    r.name.to_lowercase().contains(&query)
                }
            })
            .collect();

        // Sort by position descending (higher position first)
        filtered.sort_by_key(|r| std::cmp::Reverse(r.position));
        filtered
    });

    // Sort access roles by position (descending - higher position first)
    let sorted_access_roles = use_memo(move || {
        let mut roles = form_fields().access_roles.clone();
        let role_positions: std::collections::HashMap<i64, i16> = available_roles()
            .iter()
            .map(|r| (r.role_id, r.position))
            .collect();

        roles.sort_by_key(|ar| {
            std::cmp::Reverse(role_positions.get(&ar.role.id).copied().unwrap_or(i16::MIN))
        });
        roles
    });

    rsx! {
        div {
            class: "space-y-4",
            // Search and add role
            div {
                class: "form-control flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text", "Add Access Role" }
                }
                div {
                    class: "relative",
                    input {
                        r#type: "text",
                        class: "input input-bordered w-full",
                        placeholder: "Search roles...",
                        value: "{form_fields().role_search_query}",
                        onfocus: move |_| {
                            show_dropdown.set(true);
                        },
                        onclick: move |_| {
                            show_dropdown.set(true);
                        },
                        oninput: move |evt| {
                            form_fields.write().role_search_query = evt.value();
                            show_dropdown.set(true);
                        },
                        disabled: is_submitting,
                    }
                    // Click outside to close dropdown
                    if show_dropdown() {
                        div {
                            class: "fixed inset-0 z-0",
                            onclick: move |_| {
                                show_dropdown.set(false);
                                form_fields.write().role_search_query = String::new();
                            }
                        }
                    }
                    {
                        let roles = filtered_roles();
                        if show_dropdown() {
                            if !roles.is_empty() {
                                rsx! {
                                    div {
                                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg max-h-60 overflow-y-auto",
                                        for role in roles {
                                            {
                                                let role_color = role.color.clone();
                                                let role_name = role.name.clone();
                                                rsx! {
                                                    div {
                                                        key: "{role.role_id}",
                                                        class: "px-4 py-2 cursor-pointer hover:bg-base-200 flex items-center gap-2",
                                                        onmousedown: move |evt| {
                                                            evt.prevent_default();
                                                            let new_role = RoleData {
                                                                id: role.role_id,
                                                                name: role.name.clone(),
                                                                color: role.color.clone(),
                                                            };
                                                            let new_access_role = super::form_fields::AccessRoleData {
                                                                role: new_role,
                                                                can_view: true,
                                                                can_create: false,
                                                                can_manage: false,
                                                            };
                                                            form_fields.write().access_roles.push(new_access_role);
                                                            form_fields.write().role_search_query = String::new();
                                                            show_dropdown.set(false);
                                                        },
                                                        div {
                                                            class: "w-4 h-4 rounded",
                                                            style: "background-color: {role_color};"
                                                        }
                                                        "{role_name}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg",
                                        div {
                                            class: "px-4 py-2 text-center opacity-50 text-sm",
                                            if !form_fields().role_search_query.is_empty() {
                                                "No roles found"
                                            } else {
                                                "No roles available"
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }

            // List of access roles with scrollable container
            div {
                class: "flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text font-semibold", "Configured Access Roles" }
                }
                div {
                    class: if form_fields().access_roles.is_empty() { "space-y-2" } else { "space-y-2 max-h-96 overflow-y-auto border border-base-300 rounded-lg p-2 bg-base-200" },
                if form_fields().access_roles.is_empty() {
                    div {
                        class: "text-center py-8 opacity-50 text-sm",
                        "No access roles configured. Add roles to control who can view, create, or manage fleets in this category."
                    }
                } else {
                    for access_role in sorted_access_roles() {
                        {
                            let role_id = access_role.role.id;
                            let role_name = access_role.role.name.clone();
                            let role_color = access_role.role.color.clone();
                            let can_view = access_role.can_view;
                            let can_create = access_role.can_create;
                            let can_manage = access_role.can_manage;
                            // Find the actual index in form_fields
                            let actual_index = form_fields().access_roles.iter().position(|ar| ar.role.id == role_id).unwrap_or(0);
                            rsx! {
                                div {
                                    key: "{role_id}",
                                    class: "flex items-center gap-3 p-3 bg-base-100 rounded-lg",
                                    div {
                                        class: "w-4 h-4 rounded flex-shrink-0",
                                        style: "background-color: {role_color};"
                                    }
                                    div {
                                        class: "flex-1 font-medium",
                                        "{role_name}"
                                    }
                                    div {
                                        class: "flex gap-4",
                                        label {
                                            class: "label cursor-pointer gap-2",
                                            span { class: "label-text text-xs", "View" }
                                            input {
                                                r#type: "checkbox",
                                                class: "checkbox checkbox-sm [transition:none]",
                                                checked: can_view,
                                                disabled: is_submitting,
                                                onchange: move |evt| {
                                                    form_fields.write().access_roles[actual_index].can_view = evt.checked();
                                                }
                                            }
                                        }
                                        label {
                                            class: "label cursor-pointer gap-2",
                                            span { class: "label-text text-xs", "Create" }
                                            input {
                                                r#type: "checkbox",
                                                class: "checkbox checkbox-sm [transition:none]",
                                                checked: can_create,
                                                disabled: is_submitting,
                                                onchange: move |evt| {
                                                    form_fields.write().access_roles[actual_index].can_create = evt.checked();
                                                }
                                            }
                                        }
                                        label {
                                            class: "label cursor-pointer gap-2",
                                            span { class: "label-text text-xs", "Manage" }
                                            input {
                                                r#type: "checkbox",
                                                class: "checkbox checkbox-sm [transition:none]",
                                                checked: can_manage,
                                                disabled: is_submitting,
                                                onchange: move |evt| {
                                                    form_fields.write().access_roles[actual_index].can_manage = evt.checked();
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn-sm btn-error btn-square",
                                        disabled: is_submitting,
                                        onclick: move |_| {
                                            form_fields.write().access_roles.remove(actual_index);
                                        },
                                        "✕"
                                    }
                                }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Ping Roles tab content
#[component]
fn PingRolesTab(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let mut available_roles = use_signal(|| Vec::<DiscordGuildRoleDto>::new());

    // Fetch roles when component mounts
    #[cfg(feature = "web")]
    let roles_future =
        use_resource(move || async move { get_discord_guild_roles(guild_id, 0, 1000).await.ok() });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = roles_future.read_unchecked().as_ref() {
            available_roles.set(result.roles.clone());
        }
    });

    // Filter available roles by search query and exclude already added roles
    let filtered_roles = use_memo(move || {
        let roles = available_roles();
        let query = form_fields().role_search_query.to_lowercase();
        let ping_role_ids: Vec<i64> = form_fields().ping_roles.iter().map(|r| r.id).collect();

        let mut filtered: Vec<_> = roles
            .into_iter()
            .filter(|r| !ping_role_ids.contains(&r.role_id))
            .filter(|r| {
                if query.is_empty() {
                    true
                } else {
                    r.name.to_lowercase().contains(&query)
                }
            })
            .collect();

        // Sort by position descending (higher position first)
        filtered.sort_by_key(|r| std::cmp::Reverse(r.position));
        filtered
    });

    // Sort ping roles by position (descending - higher position first)
    let sorted_ping_roles = use_memo(move || {
        let mut roles = form_fields().ping_roles.clone();
        let role_positions: std::collections::HashMap<i64, i16> = available_roles()
            .iter()
            .map(|r| (r.role_id, r.position))
            .collect();

        roles.sort_by_key(|r| {
            std::cmp::Reverse(role_positions.get(&r.id).copied().unwrap_or(i16::MIN))
        });
        roles
    });

    rsx! {
        div {
            class: "space-y-4",
            // Search and add role
            div {
                class: "form-control flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text", "Add Ping Role" }
                }
                div {
                    class: "relative",
                    input {
                        r#type: "text",
                        class: "input input-bordered w-full",
                        placeholder: "Search roles...",
                        value: "{form_fields().role_search_query}",
                        onfocus: move |_| {
                            show_dropdown.set(true);
                        },
                        onclick: move |_| {
                            show_dropdown.set(true);
                        },
                        oninput: move |evt| {
                            form_fields.write().role_search_query = evt.value();
                            show_dropdown.set(true);
                        },
                        disabled: is_submitting,
                    }
                    // Click outside to close dropdown
                    if show_dropdown() {
                        div {
                            class: "fixed inset-0 z-0",
                            onclick: move |_| {
                                show_dropdown.set(false);
                                form_fields.write().role_search_query = String::new();
                            }
                        }
                    }
                    {
                        let roles = filtered_roles();
                        if show_dropdown() {
                            if !roles.is_empty() {
                                rsx! {
                                    div {
                                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg max-h-60 overflow-y-auto",
                                        for role in roles {
                                            {
                                                let role_color = role.color.clone();
                                                let role_name = role.name.clone();
                                                rsx! {
                                                    div {
                                                        key: "{role.role_id}",
                                                        class: "px-4 py-2 cursor-pointer hover:bg-base-200 flex items-center gap-2",
                                                        onmousedown: move |evt| {
                                                            evt.prevent_default();
                                                            let new_role = RoleData {
                                                                id: role.role_id,
                                                                name: role.name.clone(),
                                                                color: role.color.clone(),
                                                            };
                                                            form_fields.write().ping_roles.push(new_role);
                                                            form_fields.write().role_search_query = String::new();
                                                            show_dropdown.set(false);
                                                        },
                                                        div {
                                                            class: "w-4 h-4 rounded",
                                                            style: "background-color: {role_color};"
                                                        }
                                                        "{role_name}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg",
                                        div {
                                            class: "px-4 py-2 text-center opacity-50 text-sm",
                                            if !form_fields().role_search_query.is_empty() {
                                                "No roles found"
                                            } else {
                                                "No roles available"
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }

            // List of ping roles with scrollable container
            div {
                class: "flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text font-semibold", "Configured Ping Roles" }
                }
                div {
                    class: if form_fields().ping_roles.is_empty() { "space-y-2" } else { "space-y-2 max-h-96 overflow-y-auto border border-base-300 rounded-lg p-2 bg-base-200" },
                if form_fields().ping_roles.is_empty() {
                    div {
                        class: "text-center py-8 opacity-50 text-sm",
                        "No ping roles configured. Add roles to specify who gets notified about fleets in this category."
                    }
                } else {
                    for role in sorted_ping_roles() {
                        {
                            let role_id = role.id;
                            let role_name = role.name.clone();
                            let role_color = role.color.clone();
                            // Find the actual index in form_fields
                            let actual_index = form_fields().ping_roles.iter().position(|r| r.id == role_id).unwrap_or(0);
                            rsx! {
                                div {
                                    key: "{role_id}",
                                    class: "flex items-center gap-3 p-3 bg-base-100 rounded-lg",
                                    div {
                                        class: "w-4 h-4 rounded flex-shrink-0",
                                        style: "background-color: {role_color};"
                                    }
                                    div {
                                        class: "flex-1 font-medium",
                                        "{role_name}"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn-sm btn-error btn-square",
                                        disabled: is_submitting,
                                        onclick: move |_| {
                                            form_fields.write().ping_roles.remove(actual_index);
                                        },
                                        "✕"
                                    }
                                }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Channels tab content
#[component]
fn ChannelsTab(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let mut available_channels = use_signal(|| Vec::<DiscordGuildChannelDto>::new());

    // Fetch channels when component mounts
    #[cfg(feature = "web")]
    let channels_future =
        use_resource(
            move || async move { get_discord_guild_channels(guild_id, 0, 1000).await.ok() },
        );

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = channels_future.read_unchecked().as_ref() {
            available_channels.set(result.channels.clone());
        }
    });

    // Filter available channels by search query and exclude already added channels
    let filtered_channels = use_memo(move || {
        let channels = available_channels();
        let query = form_fields().channel_search_query.to_lowercase();
        let channel_ids: Vec<i64> = form_fields().channels.iter().map(|c| c.id).collect();

        channels
            .into_iter()
            .filter(|c| !channel_ids.contains(&c.channel_id))
            .filter(|c| {
                if query.is_empty() {
                    true
                } else {
                    c.name.to_lowercase().contains(&query)
                }
            })
            .collect::<Vec<_>>()
    });

    // Sort channels by position
    let sorted_channels = use_memo(move || {
        let mut channels = form_fields().channels.clone();
        let channel_positions: std::collections::HashMap<i64, i32> = available_channels()
            .iter()
            .map(|c| (c.channel_id, c.position))
            .collect();

        channels.sort_by_key(|c| channel_positions.get(&c.id).copied().unwrap_or(i32::MAX));
        channels
    });

    rsx! {
        div {
            class: "space-y-4",
            // Search and add channel
            div {
                class: "form-control flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text", "Add Channel" }
                }
                div {
                    class: "relative",
                    input {
                        r#type: "text",
                        class: "input input-bordered w-full",
                        placeholder: "Search channels...",
                        value: "{form_fields().channel_search_query}",
                        onfocus: move |_| {
                            show_dropdown.set(true);
                        },
                        onclick: move |_| {
                            show_dropdown.set(true);
                        },
                        oninput: move |evt| {
                            form_fields.write().channel_search_query = evt.value();
                            show_dropdown.set(true);
                        },
                        disabled: is_submitting,
                    }
                    // Click outside to close dropdown
                    if show_dropdown() {
                        div {
                            class: "fixed inset-0 z-0",
                            onclick: move |_| {
                                show_dropdown.set(false);
                                form_fields.write().channel_search_query = String::new();
                            }
                        }
                    }
                    {
                        let channels = filtered_channels();
                        if show_dropdown() {
                            if !channels.is_empty() {
                                rsx! {
                                    div {
                                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg max-h-60 overflow-y-auto",
                                        for channel in channels {
                                            div {
                                                key: "{channel.channel_id}",
                                                class: "px-4 py-2 cursor-pointer hover:bg-base-200",
                                                onmousedown: move |evt| {
                                                    evt.prevent_default();
                                                    let new_channel = ChannelData {
                                                        id: channel.channel_id,
                                                        name: channel.name.clone(),
                                                    };
                                                    form_fields.write().channels.push(new_channel);
                                                    form_fields.write().channel_search_query = String::new();
                                                    show_dropdown.set(false);
                                                },
                                                "# {channel.name}"
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg",
                                        div {
                                            class: "px-4 py-2 text-center opacity-50 text-sm",
                                            if !form_fields().channel_search_query.is_empty() {
                                                "No channels found"
                                            } else {
                                                "No channels available"
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }

            // List of channels with scrollable container
            div {
                class: "flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text font-semibold", "Configured Channels" }
                }
                div {
                    class: if form_fields().channels.is_empty() { "space-y-2" } else { "space-y-2 max-h-96 overflow-y-auto border border-base-300 rounded-lg p-2 bg-base-200" },
                if form_fields().channels.is_empty() {
                    div {
                        class: "text-center py-8 opacity-50 text-sm",
                        "No channels configured. Add channels where fleet notifications will be sent."
                    }
                } else {
                    for channel in sorted_channels() {
                        {
                            let channel_id = channel.id;
                            let channel_name = channel.name.clone();
                            // Find the actual index in form_fields
                            let actual_index = form_fields().channels.iter().position(|c| c.id == channel_id).unwrap_or(0);
                            rsx! {
                                div {
                                    key: "{channel_id}",
                                    class: "flex items-center gap-3 p-3 bg-base-100 rounded-lg",
                                    div {
                                        class: "flex-1 font-medium",
                                        "# {channel_name}"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn-sm btn-error btn-square",
                                        disabled: is_submitting,
                                        onclick: move |_| {
                                            form_fields.write().channels.remove(actual_index);
                                        },
                                        "✕"
                                    }
                                }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
