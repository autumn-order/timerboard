use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{
            page::{ErrorPage, LoadingPage},
            ConfirmationModal, DropdownItem, Modal, Page, SearchableDropdown,
        },
        model::error::ApiError,
        route::admin::{AdminTab, AdminTabs, AdminUsersCache},
        store::user::UserState,
    },
    model::user::UserDto,
};

#[cfg(feature = "web")]
use crate::client::api::user::{add_admin, get_all_admins, get_all_users, remove_admin};

#[component]
pub fn AdminUsers() -> Element {
    let mut cache = use_context::<Signal<AdminUsersCache>>();
    let mut error = use_signal(|| None::<ApiError>);
    let refetch_trigger = use_signal(|| 0u32);
    let mut show_add_modal = use_signal(|| false);

    // Fetch admins - resource automatically re-runs when refetch_trigger changes
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        let _ = refetch_trigger();
        get_all_admins().await
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(result) = future.read_unchecked().as_ref() {
            match result {
                Ok(admin_list) => {
                    cache.write().data = Some(admin_list.clone());
                    error.set(None);
                }
                Err(err) => {
                    tracing::error!("Failed to fetch admins: {}", err);
                    cache.write().data = None;
                    error.set(Some(err.clone()));
                }
            }
        }
    });

    rsx! {
        Title { "Admin - Users | Black Rose Timerboard" }
        if let Some(admin_list) = cache.read().data.clone() {
            Page {
                class: "flex flex-col items-center w-full h-full",
                div {
                    class: "w-full max-w-6xl",
                    h1 {
                        class: "text-lg sm:text-2xl mb-6",
                        "Admin Panel"
                    }

                    // Tabs
                    AdminTabs { active_tab: AdminTab::Users }

                    // Content
                    div {
                        class: "flex items-center justify-between gap-4 mb-6",
                        h2 {
                            class: "text-lg font-semibold",
                            "Manage Admins"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                show_add_modal.set(true);
                            },
                            "Add Admin"
                        }
                    }
                    div {
                        class: "card bg-base-200",
                        div {
                            class: "card-body",
                            AdminsList {
                                admins: admin_list.clone(),
                                refetch_trigger
                            }
                        }
                    }
                }
            }

            // Add Admin Modal
            AddAdminModal {
                show: show_add_modal,
                refetch_trigger
            }
        } else if let Some(err) = error() {
            ErrorPage { status: err.status, message: err.message }
        } else {
            LoadingPage { }
        }
    }
}

#[component]
fn AdminsList(admins: Vec<UserDto>, mut refetch_trigger: Signal<u32>) -> Element {
    let user_store = use_context::<Store<UserState>>();
    let current_user_id = user_store.read().user.as_ref().map(|u| u.discord_id);

    let mut show_delete_modal = use_signal(|| false);
    let mut admin_to_delete = use_signal(|| None::<(u64, String)>);
    let mut is_deleting = use_signal(|| false);

    // Handle deletion with use_resource
    #[cfg(feature = "web")]
    let delete_future = use_resource(move || async move {
        if is_deleting() {
            if let Some((id, _)) = admin_to_delete() {
                Some(remove_admin(id).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = delete_future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    refetch_trigger.set(refetch_trigger() + 1);
                    show_delete_modal.set(false);
                    is_deleting.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to remove admin: {}", err);
                    is_deleting.set(false);
                }
            }
        }
    });

    rsx! {
        if admins.is_empty() {
            div {
                class: "text-center py-8 opacity-50",
                "No admins found"
            }
        } else {
            div {
                class: "overflow-x-auto",
                table {
                    class: "table table-zebra w-full",
                    thead {
                        tr {
                            th { "Name" }
                            th { "Discord ID" }
                            th { class: "text-right", "Actions" }
                        }
                    }
                    tbody {
                        for admin in &admins {
                            {
                                let admin_id = admin.discord_id;
                                let admin_name = admin.name.clone();
                                let is_current_user = Some(admin_id) == current_user_id;
                                rsx! {
                                    tr {
                                        td {
                                            div {
                                                class: "flex items-center gap-2",
                                                span { "{admin.name}" }
                                                if is_current_user {
                                                    span {
                                                        class: "badge badge-sm badge-primary",
                                                        "You"
                                                    }
                                                }
                                            }
                                        }
                                        td { "{admin.discord_id}" }
                                        td {
                                            div {
                                                class: "flex gap-2 justify-end",
                                                button {
                                                    class: "btn btn-sm btn-error",
                                                    disabled: is_current_user,
                                                    onclick: move |_| {
                                                        admin_to_delete.set(Some((admin_id, admin_name.clone())));
                                                        show_delete_modal.set(true);
                                                    },
                                                    "Remove Admin"
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

        // Delete Confirmation Modal
        ConfirmationModal {
            show: show_delete_modal,
            title: "Remove Admin".to_string(),
            message: rsx!(
                if let Some((_, name)) = admin_to_delete() {
                    p {
                        class: "py-4",
                        "Are you sure you want to remove admin privileges from "
                        span { class: "font-bold", "\"{name}\"" }
                        "? They will no longer have access to admin features."
                    }
                }
            ),
            confirm_text: "Remove Admin".to_string(),
            confirm_class: "btn-error".to_string(),
            is_processing: is_deleting(),
            processing_text: "Removing...".to_string(),
            on_confirm: move |_| {
                is_deleting.set(true);
            },
        }
    }
}

#[component]
fn AddAdminModal(mut show: Signal<bool>, mut refetch_trigger: Signal<u32>) -> Element {
    let mut users_data = use_signal(|| None::<Vec<UserDto>>);
    let mut search_query = use_signal(|| String::new());
    let mut selected_user = use_signal(|| None::<u64>);
    let mut should_submit = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut show_dropdown = use_signal(|| false);

    // Reset form when modal opens
    use_effect(move || {
        if show() {
            search_query.set(String::new());
            selected_user.set(None);
            error_message.set(None);
            should_submit.set(false);
        }
    });

    // Fetch all users when modal opens (no pagination)
    #[cfg(feature = "web")]
    let fetch_future = use_resource(move || async move {
        if show() {
            // Fetch a large number to get all users
            get_all_users(0, 1000).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(data)) = fetch_future.read_unchecked().as_ref() {
            users_data.set(Some(data.users.clone()));
        }
    });

    // Handle adding admin
    #[cfg(feature = "web")]
    let add_future = use_resource(move || async move {
        if should_submit() {
            if let Some(user_id) = selected_user() {
                Some(add_admin(user_id).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = add_future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    refetch_trigger.set(refetch_trigger() + 1);
                    show.set(false);
                    should_submit.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to add admin: {}", err);
                    error_message.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    // Filter users based on search query
    let filtered_users = users_data.read().as_ref().map(|users| {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            users.clone()
        } else {
            users
                .iter()
                .filter(|user| {
                    user.name.to_lowercase().contains(&query)
                        || user.discord_id.to_string().contains(&query)
                })
                .cloned()
                .collect()
        }
    });

    // Get selected user's name for display
    let selected_user_name = if let Some(users) = users_data.read().as_ref() {
        selected_user().and_then(|id| {
            users
                .iter()
                .find(|u| u.discord_id == id)
                .map(|u| u.name.clone())
        })
    } else {
        None
    };

    let has_filtered_users = filtered_users
        .as_ref()
        .map(|u| !u.is_empty())
        .unwrap_or(false);

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        if selected_user().is_none() {
            error_message.set(Some("Please select a user".to_string()));
            return;
        }

        error_message.set(None);
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx! {
        Modal {
            show,
            title: "Add Admin".to_string(),
            prevent_close: is_submitting,
            class: "max-w-3xl ",
            form {
                class: "flex flex-col gap-4 h-full min-h-60 flex flex-col justify-between",
                onsubmit: on_submit,

                // Content section
                div {
                    class: "flex flex-col gap-4",

                    // User selection dropdown
                    div {
                    class: "form-control flex flex-col gap-2 mb-2",
                    label {
                        class: "label",
                        span {
                            class: "label-text",
                            "Select User"
                        }
                    }
                    SearchableDropdown {
                        search_query,
                        placeholder: "Search by name or Discord ID...".to_string(),
                        display_value: selected_user_name,
                        disabled: is_submitting,
                        required: true,
                        empty_message: "No users available".to_string(),
                        not_found_message: "No users found matching your search".to_string(),
                        has_items: has_filtered_users,
                        show_dropdown_signal: show_dropdown,
                        if let Some(users) = filtered_users {
                            for user in users {
                                {
                                    let user_id = user.discord_id;
                                    let user_name = user.name.clone();
                                    let user_discord_id = user.discord_id;
                                    let is_admin = user.admin;
                                    let is_selected = selected_user() == Some(user_id);
                                    rsx! {
                                        DropdownItem {
                                            key: "{user_id}",
                                            selected: is_selected,
                                            on_select: move |_| {
                                                if !is_admin {
                                                    selected_user.set(Some(user_id));
                                                    error_message.set(None);
                                                    show_dropdown.set(false);
                                                }
                                            },
                                            div {
                                                class: "flex items-center justify-between w-full",
                                                div {
                                                    class: "flex flex-col",
                                                    div {
                                                        class: "font-medium",
                                                        "{user_name}"
                                                    }
                                                    div {
                                                        class: "text-xs opacity-70",
                                                        "ID: {user_discord_id}"
                                                    }
                                                }
                                                if is_admin {
                                                    span {
                                                        class: "badge badge-sm badge-success",
                                                        "Already Admin"
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

                    // Error message
                    if let Some(err) = error_message() {
                        div {
                            class: "alert alert-error",
                            span { "{err}" }
                        }
                    }
                }

                // Actions
                div {
                    class: "modal-action",
                    button {
                        r#type: "button",
                        class: "btn",
                        disabled: is_submitting,
                        onclick: move |_| show.set(false),
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: is_submitting,
                        if is_submitting {
                            span {
                                class: "loading loading-spinner loading-sm mr-2",
                            }
                            "Adding..."
                        } else {
                            "Add Admin"
                        }
                    }
                }
            }
        }
    }
}
