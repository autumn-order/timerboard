mod component;

pub mod ping_format;

pub use ping_format::ServerAdminPingFormat;

use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{
            page::{ErrorPage, LoadingPage},
            Page,
        },
        constant::SITE_NAME,
        model::error::ApiError,
        router::Route,
    },
    model::{
        category::PaginatedFleetCategoriesDto, discord::DiscordGuildDto,
        ping_format::PaginatedPingFormatsDto,
    },
};

use component::{
    modal::CreateCategoryModal,
    table::{FleetCategoriesTable, FleetCategoryPagination},
};

/// Tab selection for the configuration section
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ConfigTab {
    #[default]
    AccessRoles,
    PingRoles,
    Channels,
}

/// Validation errors for duration fields
#[derive(Clone, Default, PartialEq)]
pub struct ValidationErrorData {
    pub ping_cooldown: Option<String>,
    pub ping_reminder: Option<String>,
    pub max_pre_ping: Option<String>,
}

#[cfg(feature = "web")]
use crate::client::api::{category::get_fleet_categories, discord_guild::get_discord_guild_by_id};

/// Cached fleet categories data for a specific guild
#[derive(Clone, PartialEq)]
pub struct FleetCategoriesCache {
    pub guild_id: u64,
    pub data: Option<PaginatedFleetCategoriesDto>,
    pub page: u64,
    pub per_page: u64,
}

/// Cached ping formats data for a specific guild
#[derive(Clone, PartialEq)]
pub struct PingFormatsCache {
    pub guild_id: u64,
    pub data: Option<PaginatedPingFormatsDto>,
    pub page: u64,
    pub per_page: u64,
}

/// Layout component that provides guild context for server admin pages
/// This layout is automatically dropped when navigating away from server admin pages,
/// which cleans up the guild context.
#[component]
pub fn ServerAdminLayout() -> Element {
    // Initialize the guild signal - it will be provided to all child routes
    // When this component is unmounted (user leaves server admin pages), the context is dropped
    use_context_provider(|| Signal::new(None::<DiscordGuildDto>));

    // Initialize fleet categories cache - persists across tab navigation within server admin
    use_context_provider(|| {
        Signal::new(FleetCategoriesCache {
            guild_id: 0,
            data: None,
            page: 0,
            per_page: 10,
        })
    });

    // Initialize ping formats cache - persists across tab navigation within server admin
    use_context_provider(|| {
        Signal::new(PingFormatsCache {
            guild_id: 0,
            data: None,
            page: 0,
            per_page: 10,
        })
    });

    rsx! {
        // Render child routes (ServerAdminFleetCategory or ServerAdminPingFormat)
        Outlet::<Route> {}
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ServerAdminTab {
    FleetCategories,
    PingFormats,
}

#[component]
pub fn GuildInfoHeader(guild_data: DiscordGuildDto) -> Element {
    rsx!(
        // Header with guild info
        div {
            class: "card bg-base-200 mb-8",
            div {
                class: "card-body",
                div {
                    class: "flex items-center gap-4",
                    if let Some(icon_hash) = &guild_data.icon_hash {
                        img {
                            src: "https://cdn.discordapp.com/icons/{guild_data.guild_id}/{icon_hash}.png",
                            alt: "{guild_data.name} icon",
                            class: "w-16 h-16 rounded-full",
                        }
                    } else {
                        div {
                            class: "w-16 h-16 rounded-full bg-neutral flex items-center justify-center font-bold text-2xl",
                            "{guild_data.name.chars().next().unwrap_or('?')}"
                        }
                    }
                    div {
                        class: "flex-1",
                        h1 {
                            class: "text-2xl font-bold",
                            "{guild_data.name}"
                        }
                        p {
                            class: "text-sm opacity-70",
                            "Guild ID: {guild_data.guild_id}"
                        }
                    }
                }
            }
        }
    )
}

#[component]
pub fn ActionTabs(guild_id: u64, active_tab: ServerAdminTab) -> Element {
    rsx! (
        div {
            role: "tablist",
            class: "tabs tabs-bordered mb-6",
            Link {
                to: Route::ServerAdminFleetCategory { guild_id },
                role: "tab",
                class: if active_tab == ServerAdminTab::FleetCategories { "tab tab-active" } else { "tab" },
                "Fleet Categories"
            }
            Link {
                to: Route::ServerAdminPingFormat { guild_id },
                role: "tab",
                class: if active_tab == ServerAdminTab::PingFormats { "tab tab-active" } else { "tab" },
                "Ping Formats"
            }
        }
    )
}

#[component]
pub fn ServerAdminFleetCategory(guild_id: u64) -> Element {
    let mut guild = use_context::<Signal<Option<DiscordGuildDto>>>();
    let mut error = use_signal(|| None::<ApiError>);

    // Fetch guild data using use_resource if not already cached
    #[cfg(feature = "web")]
    {
        let mut should_fetch = use_signal(|| false);

        // Check cache and initiate fetch if needed
        use_effect(use_reactive!(|guild_id| {
            // Skip if already fetching
            if should_fetch() {
                return;
            }

            // Only run resource if we need to fetch
            let needs_fetch = guild.read().as_ref().map(|g| g.guild_id) != Some(guild_id);

            if needs_fetch {
                should_fetch.set(true);
            }
        }));

        let future = use_resource(move || async move {
            if should_fetch() {
                Some(get_discord_guild_by_id(guild_id).await)
            } else {
                None
            }
        });

        use_effect(move || {
            if let Some(Some(result)) = future.read_unchecked().as_ref() {
                match result {
                    Ok(guild_data) => {
                        guild.set(Some(guild_data.clone()));
                        error.set(None);
                    }
                    Err(err) => {
                        tracing::error!("Failed to fetch guild: {}", err);
                        guild.set(None);
                        error.set(Some(err.clone()));
                    }
                }
            }
        });
    }

    rsx! {
        Title { "Fleet Categories | {SITE_NAME}" }
        if let Some(guild_data) = guild.read().clone() {
            Page {
                class: "flex flex-col items-center w-full h-full",
                div {
                    class: "w-full max-w-6xl",
                    Link {
                        to: Route::AdminServers {},
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
