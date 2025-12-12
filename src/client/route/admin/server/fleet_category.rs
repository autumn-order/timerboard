use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{
            page::{ErrorPage, LoadingPage},
            ConfirmationModal, Modal, Page, Pagination, PaginationData,
        },
        model::error::ApiError,
        route::admin::server::{ActionTabs, FleetCategoriesCache, GuildInfoHeader, ServerAdminTab},
        router::Route,
    },
    model::{discord::DiscordGuildDto, fleet::PaginatedFleetCategoriesDto},
};

#[cfg(feature = "web")]
use crate::client::api::{
    discord_guild::get_discord_guild_by_id,
    fleet_category::{
        create_fleet_category, delete_fleet_category, get_fleet_categories, update_fleet_category,
    },
};

#[component]
pub fn ServerAdminFleetCategory(guild_id: u64) -> Element {
    let mut guild = use_context::<Signal<Option<DiscordGuildDto>>>();
    let mut error = use_signal(|| None::<ApiError>);

    // Fetch guild data using use_resource if not already cached
    #[cfg(feature = "web")]
    {
        // Only run resource if we need to fetch
        let needs_fetch = guild.read().as_ref().map(|g| g.guild_id as u64) != Some(guild_id);

        if needs_fetch {
            let future =
                use_resource(move || async move { get_discord_guild_by_id(guild_id).await });

            match &*future.read_unchecked() {
                Some(Ok(guild_data)) => {
                    guild.set(Some(guild_data.clone()));
                    error.set(None);
                }
                Some(Err(err)) => {
                    tracing::error!("Failed to fetch guild: {}", err);
                    guild.set(None);
                    error.set(Some(err.clone()));
                }
                None => (),
            }
        }
    }

    rsx! {
        Title { "Fleet Categories | Black Rose Timerboard" }
        if let Some(guild_data) = guild.read().clone() {
            Page {
                class: "flex flex-col items-center w-full h-full",
                div {
                    class: "w-full max-w-6xl",
                    Link {
                        to: Route::Admin {},
                        class: "btn btn-ghost mb-4",
                        "← Back to Servers"
                    }
                    GuildInfoHeader { guild_data: guild_data.clone() }
                    ActionTabs { guild_id, active_tab: ServerAdminTab::FleetCategories }
                    div {
                        class: "space-y-6",
                        FleetCategoriesSection { guild_id }
                    }
                }
            }
        } else if let Some(err) = error() {
            ErrorPage { status: err.status, message: err.message }
        } else {
            LoadingPage { }
        }
    }
}

#[component]
fn FleetCategoriesSection(guild_id: u64) -> Element {
    let mut cache = use_context::<Signal<FleetCategoriesCache>>();
    let mut error = use_signal(|| None::<ApiError>);
    let mut show_create_modal = use_signal(|| false);

    // Get page and per_page from cache
    let page = use_signal(|| cache.read().page);
    let per_page = use_signal(|| cache.read().per_page);
    let refetch_trigger = use_signal(|| 0u32);

    // Fetch fleet categories - resource automatically re-runs when page(), per_page(), or refetch_trigger changes
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        let _ = refetch_trigger();
        get_fleet_categories(guild_id, page(), per_page()).await
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(result) = future.read_unchecked().as_ref() {
            match result {
                Ok(data) => {
                    // Update cache
                    cache.write().guild_id = guild_id;
                    cache.write().data = Some(data.clone());
                    cache.write().page = page();
                    cache.write().per_page = per_page();
                    error.set(None);
                }
                Err(err) => {
                    tracing::error!("Failed to fetch fleet categories: {}", err);
                    cache.write().data = None;
                    error.set(Some(err.clone()));
                }
            }
        }
    });

    rsx!(
        div {
            class: "card bg-base-200",
            div {
                class: "card-body",
                div {
                    class: "flex justify-between items-center mb-4",
                    h2 {
                        class: "card-title",
                        "Fleet Categories"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create_modal.set(true),
                        "Add Category"
                    }
                }

                // Content
                if let Some(data) = cache.read().data.clone() {
                    if data.categories.is_empty() {
                        div {
                            class: "text-center py-8 opacity-50",
                            "No fleet categories configured"
                        }
                    } else {
                        FleetCategoriesTable {
                            data: data.clone(),
                            guild_id,
                            cache,
                            refetch_trigger
                        }
                        FleetCategoryPagination {
                            page,
                            per_page,
                            pagination_data: data.clone(),
                            cache
                        }
                    }
                } else if let Some(err) = error() {
                    div {
                        class: "alert alert-error",
                        span { "Error loading categories: {err.message}" }
                    }
                } else {
                    div {
                        class: "text-center py-8",
                        span { class: "loading loading-spinner loading-lg" }
                    }
                }

                // Create Category Modal
                CreateCategoryModal {
                    guild_id,
                    show: show_create_modal,
                    refetch_trigger
                }
            }
        }
    )
}

/// Helper to format duration to string like "1h", "30m", "1h30m"
fn format_duration(d: &Duration) -> String {
    let total_seconds = d.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 && minutes > 0 {
        format!("{}h{}m", hours, minutes)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", total_seconds)
    }
}

/// Helper to parse duration string like "1h", "30m", "2h30m"
fn parse_duration(s: &str) -> Option<Duration> {
    if s.trim().is_empty() {
        return None;
    }

    let s = s.trim().to_lowercase();
    let mut total_seconds = 0i64;
    let mut current_num = String::new();

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current_num.push(ch);
        } else if ch == 'h' {
            if let Ok(hours) = current_num.parse::<i64>() {
                total_seconds += hours * 3600;
                current_num.clear();
            }
        } else if ch == 'm' {
            if let Ok(minutes) = current_num.parse::<i64>() {
                total_seconds += minutes * 60;
                current_num.clear();
            }
        } else if ch == 's' {
            if let Ok(seconds) = current_num.parse::<i64>() {
                total_seconds += seconds;
                current_num.clear();
            }
        }
    }

    if total_seconds > 0 {
        Some(Duration::seconds(total_seconds))
    } else {
        None
    }
}

/// Reusable form fields component for fleet category forms
#[component]
fn FleetCategoryFormFields(
    category_name: Signal<String>,
    ping_cooldown_str: Signal<String>,
    ping_reminder_str: Signal<String>,
    max_pre_ping_str: Signal<String>,
    is_submitting: bool,
) -> Element {
    rsx! {
        // Category Name Input
        div {
            class: "form-control w-full gap-3",
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
                value: "{category_name()}",
                oninput: move |evt| category_name.set(evt.value()),
                disabled: is_submitting,
                required: true,
            }
        }

        // Ping Cooldown Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Ping Cooldown (optional)"
                }
            }
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "e.g., 1h, 30m, 1h30m",
                value: "{ping_cooldown_str()}",
                oninput: move |evt| ping_cooldown_str.set(evt.value()),
                disabled: is_submitting,
            }
            label {
                class: "label flex-col items-start gap-1",
                span {
                    class: "label-text-alt",
                    "Minimum amount of time between fleets"
                }
                span {
                    class: "label-text-alt text-xs",
                    "Format: 1h = 1 hour, 30m = 30 minutes, 1h30m = 1.5 hours"
                }
            }
        }

        // Ping Reminder Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Ping Reminder (optional)"
                }
            }
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "e.g., 15m, 30m",
                value: "{ping_reminder_str()}",
                oninput: move |evt| ping_reminder_str.set(evt.value()),
                disabled: is_submitting,
            }
            label {
                class: "label flex-col items-start gap-1",
                span {
                    class: "label-text-alt",
                    "Reminder ping before fleet starts"
                }
                span {
                    class: "label-text-alt text-xs",
                    "Format: 1h = 1 hour, 30m = 30 minutes, 1h30m = 1.5 hours"
                }
            }
        }

        // Max Pre-Ping Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Max Pre-Ping (optional)"
                }
            }
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "e.g., 2h, 3h",
                value: "{max_pre_ping_str()}",
                oninput: move |evt| max_pre_ping_str.set(evt.value()),
                disabled: is_submitting,
            }
            label {
                class: "label flex-col items-start gap-1",
                span {
                    class: "label-text-alt",
                    "Maximum advance notice for pings"
                }
                span {
                    class: "label-text-alt text-xs",
                    "Format: 1h = 1 hour, 30m = 30 minutes, 1h30m = 1.5 hours"
                }
            }
        }
    }
}

#[component]
fn CreateCategoryModal(
    guild_id: u64,
    mut show: Signal<bool>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut category_name = use_signal(|| String::new());
    let mut ping_cooldown_str = use_signal(|| String::new());
    let mut ping_reminder_str = use_signal(|| String::new());
    let mut max_pre_ping_str = use_signal(|| String::new());
    let mut submit_name = use_signal(|| String::new());
    let mut submit_ping_cooldown = use_signal(|| None::<Duration>);
    let mut submit_ping_reminder = use_signal(|| None::<Duration>);
    let mut submit_max_pre_ping = use_signal(|| None::<Duration>);
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Reset form when modal opens (clears data from previous use)
    use_effect(move || {
        if show() {
            category_name.set(String::new());
            ping_cooldown_str.set(String::new());
            ping_reminder_str.set(String::new());
            max_pre_ping_str.set(String::new());
            submit_name.set(String::new());
            submit_ping_cooldown.set(None);
            submit_ping_reminder.set(None);
            submit_max_pre_ping.set(None);
            should_submit.set(false);
            error.set(None);
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            Some(
                create_fleet_category(
                    guild_id,
                    submit_name(),
                    submit_ping_cooldown(),
                    submit_ping_reminder(),
                    submit_max_pre_ping(),
                )
                .await,
            )
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Close modal (data persists for smooth animation)
                    show.set(false);
                    should_submit.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to create category: {}", err);
                    error.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let name = category_name();
        if name.trim().is_empty() {
            error.set(Some("Category name is required".to_string()));
            return;
        }

        error.set(None);
        submit_name.set(name);
        submit_ping_cooldown.set(parse_duration(&ping_cooldown_str()));
        submit_ping_reminder.set(parse_duration(&ping_reminder_str()));
        submit_max_pre_ping.set(parse_duration(&max_pre_ping_str()));
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        Modal {
            show,
            title: "Create Fleet Category".to_string(),
            prevent_close: is_submitting,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                FleetCategoryFormFields {
                    category_name,
                    ping_cooldown_str,
                    ping_reminder_str,
                    max_pre_ping_str,
                    is_submitting
                }

                // Error Message
                if let Some(err) = error() {
                    div {
                        class: "alert alert-error mt-4",
                        span { "{err}" }
                    }
                }

                // Modal Actions
                div {
                    class: "modal-action",
                    button {
                        r#type: "button",
                        class: "btn",
                        onclick: move |_| show.set(false),
                        disabled: is_submitting,
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: is_submitting,
                        if is_submitting {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Creating..."
                        } else {
                            "Create"
                        }
                    }
                }
            }
        }
    )
}

#[component]
fn FleetCategoriesTable(
    data: PaginatedFleetCategoriesDto,
    guild_id: u64,
    mut cache: Signal<FleetCategoriesCache>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut sorted_categories = data.categories.clone();
    sorted_categories.sort_by_key(|c| c.id);

    let mut show_delete_modal = use_signal(|| false);
    let mut category_to_delete = use_signal(|| None::<(i32, String)>);
    let mut is_deleting = use_signal(|| false);

    let mut show_edit_modal = use_signal(|| false);
    let mut category_to_edit = use_signal(|| None::<crate::model::fleet::FleetCategoryDto>);

    // Handle deletion with use_resource
    #[cfg(feature = "web")]
    let delete_future = use_resource(move || async move {
        if is_deleting() {
            if let Some((id, _)) = category_to_delete() {
                Some(delete_fleet_category(guild_id, id).await)
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
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Close modal (data persists for smooth animation)
                    show_delete_modal.set(false);
                    is_deleting.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to delete category: {}", err);
                    is_deleting.set(false);
                }
            }
        }
    });

    rsx!(
        div {
            class: "overflow-x-auto",
            table {
                class: "table table-zebra w-full",
                thead {
                    tr {
                        th { "Name" }
                        th { class: "text-center", "Upcoming Fleets" }
                        th { class: "text-center", "All-time Total" }
                        th { class: "text-center", "Configured Roles" }
                        th {
                            class: "text-right",
                            "Actions"
                        }
                    }
                }
                tbody {
                    for category in &sorted_categories {
                        {
                            let category_id = category.id;
                            let category_name = category.name.clone();
                            let category_clone_for_edit = category.clone();
                            let category_name_for_delete = category_name.clone();
                            rsx! {
                                tr {
                                    td { "{category.name}" }
                                    td { class: "text-center", "0" }
                                    td { class: "text-center", "0" }
                                    td { class: "text-center", "0" }
                                    td {
                                        div {
                                            class: "flex gap-2 justify-end",
                                            button {
                                                class: "btn btn-sm btn-primary",
                                                onclick: move |_| {
                                                    category_to_edit.set(Some(category_clone_for_edit.clone()));
                                                    show_edit_modal.set(true);
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-sm btn-error",
                                                onclick: move |_| {
                                                    category_to_delete.set(Some((category_id, category_name_for_delete.clone())));
                                                    show_delete_modal.set(true);
                                                },
                                                "Delete"
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
            title: "Delete Fleet Category".to_string(),
            message: rsx!(
                if let Some((_, name)) = category_to_delete() {
                    p {
                        class: "py-4",
                        "Are you sure you want to delete the category "
                        span { class: "font-bold", "\"{name}\"" }
                        "? This action cannot be undone."
                    }
                }
            ),
            confirm_text: "Delete".to_string(),
            confirm_class: "btn-error".to_string(),
            is_processing: is_deleting(),
            processing_text: "Deleting...".to_string(),
            on_confirm: move |_| {
                is_deleting.set(true);
            },
        }

        // Edit Category Modal
        EditCategoryModal {
            guild_id,
            show: show_edit_modal,
            category_to_edit,
            refetch_trigger
        }
    )
}

#[component]
fn EditCategoryModal(
    guild_id: u64,
    mut show: Signal<bool>,
    category_to_edit: Signal<Option<crate::model::fleet::FleetCategoryDto>>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut category_name = use_signal(|| String::new());
    let mut ping_cooldown_str = use_signal(|| String::new());
    let mut ping_reminder_str = use_signal(|| String::new());
    let mut max_pre_ping_str = use_signal(|| String::new());
    let mut submit_name = use_signal(|| String::new());
    let mut submit_ping_cooldown = use_signal(|| None::<Duration>);
    let mut submit_ping_reminder = use_signal(|| None::<Duration>);
    let mut submit_max_pre_ping = use_signal(|| None::<Duration>);
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut category_id = use_signal(|| 0i32);

    // Initialize form when modal opens with new data
    use_effect(move || {
        if show() {
            if let Some(category) = category_to_edit() {
                category_name.set(category.name.clone());
                category_id.set(category.id);
                ping_cooldown_str.set(
                    category
                        .ping_lead_time
                        .as_ref()
                        .map(|d| format_duration(d))
                        .unwrap_or_default(),
                );
                ping_reminder_str.set(
                    category
                        .ping_reminder
                        .as_ref()
                        .map(|d| format_duration(d))
                        .unwrap_or_default(),
                );
                max_pre_ping_str.set(
                    category
                        .max_pre_ping
                        .as_ref()
                        .map(|d| format_duration(d))
                        .unwrap_or_default(),
                );
                // Reset error and submit state when opening with new data
                error.set(None);
                should_submit.set(false);
            }
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            Some(
                update_fleet_category(
                    guild_id,
                    category_id(),
                    submit_name(),
                    submit_ping_cooldown(),
                    submit_ping_reminder(),
                    submit_max_pre_ping(),
                )
                .await,
            )
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Close modal (data persists for smooth animation)
                    show.set(false);
                    should_submit.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to update category: {}", err);
                    error.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let name = category_name();
        if name.trim().is_empty() {
            error.set(Some("Category name is required".to_string()));
            return;
        }

        error.set(None);
        submit_name.set(name);
        submit_ping_cooldown.set(parse_duration(&ping_cooldown_str()));
        submit_ping_reminder.set(parse_duration(&ping_reminder_str()));
        submit_max_pre_ping.set(parse_duration(&max_pre_ping_str()));
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        Modal {
            show,
            title: "Edit Fleet Category".to_string(),
            prevent_close: is_submitting,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                FleetCategoryFormFields {
                    category_name,
                    ping_cooldown_str,
                    ping_reminder_str,
                    max_pre_ping_str,
                    is_submitting
                }

                // Error Message
                if let Some(err) = error() {
                    div {
                        class: "alert alert-error mt-4",
                        span { "{err}" }
                    }
                }

                // Modal Actions
                div {
                    class: "modal-action",
                    button {
                        r#type: "button",
                        class: "btn",
                        onclick: move |_| show.set(false),
                        disabled: is_submitting,
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: is_submitting,
                        if is_submitting {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Updating..."
                        } else {
                            "Update"
                        }
                    }
                }
            }
        }
    )
}

#[component]
fn FleetCategoryPagination(
    mut page: Signal<u64>,
    mut per_page: Signal<u64>,
    pagination_data: PaginatedFleetCategoriesDto,
    mut cache: Signal<FleetCategoriesCache>,
) -> Element {
    let data = PaginationData {
        page: pagination_data.page,
        per_page: pagination_data.per_page,
        total: pagination_data.total,
        total_pages: pagination_data.total_pages,
    };

    rsx!(Pagination {
        page,
        per_page,
        data,
        on_page_change: move |new_page| {
            cache.write().page = new_page;
        },
        on_per_page_change: move |new_per_page| {
            cache.write().per_page = new_per_page;
            cache.write().page = 0;
        },
    })
}
