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

use super::{ActionTabs, GuildInfoHeader, ServerAdminTab};

#[component]
pub fn ServerAdminFleetCategory(guild_id: u64) -> Element {
    let mut guild = use_context::<Signal<Option<DiscordGuildDto>>>();
    let mut error = use_signal(|| None::<ApiError>);

    // Fetch guild data using use_resource if not already cached
    #[cfg(feature = "web")]
    {
        use crate::client::route::admin::get_discord_guild_by_id;

        let future = use_resource(move || async move {
            // Only fetch if we don't have the guild data or if the guild_id doesn't match
            if guild.read().as_ref().map(|g| g.guild_id as u64) != Some(guild_id) {
                get_discord_guild_by_id(guild_id).await
            } else {
                // Return a dummy error to skip updating
                Err(ApiError {
                    status: 0,
                    message: "cached".to_string(),
                })
            }
        });

        match &*future.read_unchecked() {
            Some(Ok(guild_data)) => {
                guild.set(Some(guild_data.clone()));
                error.set(None);
            }
            Some(Err(err)) if err.status != 0 => {
                tracing::error!("Failed to fetch guild: {}", err);
                guild.set(None);
                error.set(Some(err.clone()));
            }
            _ => (),
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
    let page = use_signal(|| 0u64);
    let per_page = use_signal(|| 10u64);
    let mut categories_data = use_signal(|| None::<Result<PaginatedFleetCategoriesDto, ApiError>>);

    // Fetch fleet categories
    #[cfg(feature = "web")]
    {
        let future = use_resource(move || async move {
            get_fleet_categories(guild_id, page(), per_page()).await
        });

        match &*future.read_unchecked() {
            Some(Ok(data)) => {
                categories_data.set(Some(Ok(data.clone())));
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch fleet categories: {}", err);
                categories_data.set(Some(Err(err.clone())));
            }
            None => (),
        }
    }

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
                        "Add Category"
                    }
                }

                // Content
                if let Some(Ok(data)) = categories_data() {
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
                            pagination_data: data.clone()
                        }
                    }
                } else if let Some(Err(err)) = categories_data() {
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
            }
        }
    )
}

#[component]
fn FleetCategoriesTable(data: PaginatedFleetCategoriesDto) -> Element {
    rsx!(
        div {
            class: "overflow-x-auto",
            table {
                class: "table table-zebra w-full",
                thead {
                    tr {
                        th { "ID" }
                        th { "Name" }
                        th { "Actions" }
                    }
                }
                tbody {
                    for category in &data.categories {
                        tr {
                            td { "{category.id}" }
                            td { "{category.name}" }
                            td {
                                div {
                                    class: "flex gap-2",
                                    button {
                                        class: "btn btn-sm btn-ghost",
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
                                page.set(page() - 1);
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
                                page.set(page() + 1);
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
