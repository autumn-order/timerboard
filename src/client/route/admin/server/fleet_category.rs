use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::client::{
    component::{
        page::{ErrorPage, LoadingPage},
        Page,
    },
    model::error::ApiError,
    router::Route,
};

use crate::model::{discord::DiscordGuildDto, fleet::PaginatedFleetCategoriesDto};

use super::{ActionTabs, FleetCategoriesCache, GuildInfoHeader, ServerAdminTab};

#[component]
pub fn ServerAdminFleetCategory(guild_id: u64) -> Element {
    let mut guild = use_context::<Signal<Option<DiscordGuildDto>>>();
    let mut error = use_signal(|| None::<ApiError>);

    // Fetch guild data using use_resource if not already cached
    #[cfg(feature = "web")]
    {
        use crate::client::route::admin::get_discord_guild_by_id;

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

    // Fetch fleet categories - resource automatically re-runs when page() or per_page() changes
    #[cfg(feature = "web")]
    let future =
        use_resource(
            move || async move { get_fleet_categories(guild_id, page(), per_page()).await },
        );

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
                        FleetCategoriesTable { data: data.clone() }
                        Pagination {
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
                    cache
                }
            }
        }
    )
}

#[component]
fn CreateCategoryModal(
    guild_id: u64,
    mut show: Signal<bool>,
    mut cache: Signal<FleetCategoriesCache>,
) -> Element {
    let mut category_name = use_signal(|| String::new());
    let mut submit_name = use_signal(|| String::new());
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Reset form when modal is closed
    use_effect(move || {
        if !show() {
            category_name.set(String::new());
            submit_name.set(String::new());
            should_submit.set(false);
            error.set(None);
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            Some(create_fleet_category(guild_id, submit_name()).await)
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    // Clear cache to force refetch
                    cache.write().data = None;
                    // Close modal
                    show.set(false);
                    // Reset form
                    category_name.set(String::new());
                    submit_name.set(String::new());
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
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        // DaisyUI Modal
        div {
            class: if show() { "modal modal-open" } else { "modal" },
            div {
                class: "modal-box",
                h3 {
                    class: "font-bold text-lg mb-4",
                    "Create Fleet Category"
                }

                form {
                    onsubmit: on_submit,

                    // Category Name Input
                    div {
                        class: "form-control w-full flex flex-col gap-3",
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
            // Modal backdrop
            div {
                class: "modal-backdrop",
                onclick: move |_| {
                    if !is_submitting {
                        show.set(false);
                    }
                },
            }
        }
    )
}

#[component]
fn FleetCategoriesTable(data: PaginatedFleetCategoriesDto) -> Element {
    let mut sorted_categories = data.categories.clone();
    sorted_categories.sort_by_key(|c| c.id);

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
                                        "Edit"
                                    }
                                    button {
                                        class: "btn btn-sm btn-error",
                                        "Delete"
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
fn Pagination(
    mut page: Signal<u64>,
    mut per_page: Signal<u64>,
    pagination_data: PaginatedFleetCategoriesDto,
    mut cache: Signal<FleetCategoriesCache>,
) -> Element {
    rsx!(
        div {
            class: "flex justify-between items-center mt-4",
            // Per-page selector
            div {
                class: "flex items-center gap-2",
                span { "Show" }
                select {
                    class: "select select-bordered select-sm",
                    value: "{per_page()}",
                    onchange: move |evt| {
                        if let Ok(value) = evt.value().parse::<u64>() {
                            per_page.set(value);
                            page.set(0); // Reset to first page
                            // Update cache
                            cache.write().per_page = value;
                            cache.write().page = 0;
                        }
                    },
                    option { value: "5", "5" }
                    option { value: "10", "10" }
                    option { value: "25", "25" }
                    option { value: "50", "50" }
                    option { value: "100", "100" }
                }
                span { "entries" }
            }

            // Pagination info and buttons
            div {
                class: "flex items-center gap-4",
                span {
                    class: "text-sm opacity-70",
                    "Showing {(pagination_data.page * pagination_data.per_page) + 1} to {((pagination_data.page + 1) * pagination_data.per_page).min(pagination_data.total)} of {pagination_data.total}"
                }
                div {
                    class: "join",
                    button {
                        class: "join-item btn btn-sm",
                        disabled: pagination_data.page == 0,
                        onclick: move |_| {
                            if page() > 0 {
                                let new_page = page() - 1;
                                page.set(new_page);
                                cache.write().page = new_page;
                            }
                        },
                        "«"
                    }
                    button {
                        class: "join-item btn btn-sm",
                        "Page {pagination_data.page + 1} of {pagination_data.total_pages}"
                    }
                    button {
                        class: "join-item btn btn-sm",
                        disabled: pagination_data.page >= pagination_data.total_pages - 1,
                        onclick: move |_| {
                            if page() < pagination_data.total_pages - 1 {
                                let new_page = page() + 1;
                                page.set(new_page);
                                cache.write().page = new_page;
                            }
                        },
                        "»"
                    }
                }
            }
        }
    )
}

#[cfg(feature = "web")]
async fn get_fleet_categories(
    guild_id: u64,
    page: u64,
    per_page: u64,
) -> Result<PaginatedFleetCategoriesDto, ApiError> {
    use crate::model::api::ErrorDto;
    use reqwasm::http::Request;

    let url = format!(
        "/api/timerboard/{}/fleet/category?page={}&entries={}",
        guild_id, page, per_page
    );

    let response = Request::get(&url)
        .credentials(reqwasm::http::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 500,
            message: format!("Failed to send request: {}", e),
        })?;

    let status = response.status() as u64;

    match status {
        200 => {
            let data = response
                .json::<PaginatedFleetCategoriesDto>()
                .await
                .map_err(|e| ApiError {
                    status: 500,
                    message: format!("Failed to parse fleet categories: {}", e),
                })?;
            Ok(data)
        }
        _ => {
            let message = if let Ok(error_dto) = response.json::<ErrorDto>().await {
                error_dto.error
            } else {
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string())
            };

            Err(ApiError { status, message })
        }
    }
}

#[cfg(feature = "web")]
async fn create_fleet_category(guild_id: u64, name: String) -> Result<(), ApiError> {
    use crate::model::{api::ErrorDto, fleet::CreateFleetCategoryDto};
    use reqwasm::http::Request;

    let url = format!("/api/timerboard/{}/fleet/category", guild_id);

    let payload = CreateFleetCategoryDto { name };

    let response = Request::post(&url)
        .credentials(reqwasm::http::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&payload).map_err(|e| ApiError {
            status: 500,
            message: format!("Failed to serialize request: {}", e),
        })?)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 500,
            message: format!("Failed to send request: {}", e),
        })?;

    let status = response.status() as u64;

    match status {
        201 => Ok(()),
        _ => {
            let message = if let Ok(error_dto) = response.json::<ErrorDto>().await {
                error_dto.error
            } else {
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string())
            };

            Err(ApiError { status, message })
        }
    }
}
