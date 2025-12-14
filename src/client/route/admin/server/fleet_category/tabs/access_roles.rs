use dioxus::prelude::*;

use crate::client::component::{DropdownItem, SearchableDropdown, SelectedItem, SelectedItemsList};
use crate::model::discord::DiscordGuildRoleDto;

use super::super::types::{AccessRoleData, FormFieldsData, RoleData};

#[cfg(feature = "web")]
use crate::client::api::discord::get_discord_guild_roles;

#[component]
pub fn AccessRolesTab(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
) -> Element {
    let mut available_roles = use_signal(|| Vec::<DiscordGuildRoleDto>::new());
    let mut role_search_query = use_signal(|| String::new());
    let mut role_dropdown_open = use_signal(|| false);
    let mut should_fetch_roles = use_signal(|| false);

    // Fetch roles only when dropdown is focused
    #[cfg(feature = "web")]
    let roles_future = use_resource(move || async move {
        if should_fetch_roles() {
            get_discord_guild_roles(guild_id, 0, 1000).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = roles_future.read_unchecked().as_ref() {
            available_roles.set(result.roles.clone());
        }
    });

    // Handler for when dropdown is focused
    let on_dropdown_focus = move |_| {
        if available_roles().is_empty() {
            should_fetch_roles.set(true);
        }
    };

    // Filter available roles by search query and exclude already added roles
    let filtered_roles = use_memo(move || {
        let roles = available_roles();
        let query = role_search_query().to_lowercase();
        let access_role_ids: Vec<u64> = form_fields()
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
        roles.sort_by_key(|ar| std::cmp::Reverse(ar.role.position));
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
                SearchableDropdown {
                    search_query: role_search_query,
                    placeholder: "Search roles...".to_string(),
                    display_value: None,
                    disabled: is_submitting,
                    has_items: !filtered_roles().is_empty(),
                    show_dropdown_signal: Some(role_dropdown_open),
                    on_focus: on_dropdown_focus,
                    for role in filtered_roles() {
                        {
                            let role_name = role.name.clone();
                            let role_color = role.color.clone();
                            rsx! {
                                DropdownItem {
                                    key: "{role.role_id}",
                                    on_select: move |_| {
                                        let new_role = RoleData {
                                            id: role.role_id,
                                            name: role.name.clone(),
                                            color: role.color.clone(),
                                            position: role.position,
                                        };
                                        let new_access_role = AccessRoleData {
                                            role: new_role,
                                            can_view: true,
                                            can_create: false,
                                            can_manage: false,
                                        };
                                        form_fields.write().access_roles.push(new_access_role);
                                        role_search_query.set(String::new());
                                        role_dropdown_open.set(false);
                                    },
                                    div {
                                        class: "flex items-center gap-2",
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
            }

            // List of access roles with scrollable container
            SelectedItemsList {
                label: "Configured Access Roles".to_string(),
                empty_message: "No access roles configured. Add roles to control who can view, create, or manage fleets in this category.".to_string(),
                is_empty: sorted_access_roles().is_empty(),
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
                            SelectedItem {
                                key: "{role_id}",
                                disabled: is_submitting,
                                on_remove: move |_| {
                                    form_fields.write().access_roles.remove(actual_index);
                                },
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
                            }
                        }
                    }
                }
            }
        }
    }
}
