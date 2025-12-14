pub mod fleet_category;
pub mod ping_format;

pub use fleet_category::ServerAdminFleetCategory;
pub use ping_format::ServerAdminPingFormat;

use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{
            page::{ErrorPage, LoadingPage},
            Page,
        },
        model::error::ApiError,
        router::Route,
    },
    model::{
        category::PaginatedFleetCategoriesDto, discord::DiscordGuildDto,
        ping_format::PaginatedPingFormatsDto,
    },
};

#[cfg(feature = "web")]
use crate::client::api::discord_guild::get_discord_guild_by_id;

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
        // Render child routes (ServerAdmin or ServerAdminFleetCategory)
        Outlet::<Route> {}
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ServerAdminTab {
    Overview,
    FleetCategories,
    PingFormats,
}

#[component]
pub fn ServerAdmin(guild_id: u64) -> Element {
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
        Title { "Server Admin | Black Rose Timerboard" }
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
                    ActionTabs { guild_id, active_tab: ServerAdminTab::Overview }
                    div {
                        class: "space-y-6",
                        OverviewSection { }
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
                to: Route::ServerAdmin { guild_id },
                role: "tab",
                class: if active_tab == ServerAdminTab::Overview { "tab tab-active" } else { "tab" },
                "Overview"
            }
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
fn OverviewSection() -> Element {
    rsx!(
        div {
            class: "card bg-base-200",
            div {
                class: "card-body",
                h2 {
                    class: "card-title",
                    "Timerboard Overview"
                }
                div {
                    class: "stats stats-vertical lg:stats-horizontal",
                    div {
                        class: "stat",
                        div {
                            class: "stat-title",
                            "Upcoming Timers"
                        }
                        div {
                            class: "stat-value",
                            "0"
                        }
                        div {
                            class: "stat-desc",
                            "No upcoming timers"
                        }
                    }
                    div {
                        class: "stat",
                        div {
                            class: "stat-title",
                            "Total Timers"
                        }
                        div {
                            class: "stat-value",
                            "0"
                        }
                        div {
                            class: "stat-desc",
                            "All-time total"
                        }
                    }
                    div {
                        class: "stat",
                        div {
                            class: "stat-title",
                            "Fleet Categories"
                        }
                        div {
                            class: "stat-value",
                            "0"
                        }
                        div {
                            class: "stat-desc",
                            "Manage categories"
                        }
                    }
                }
            }
        }

        // Recent activity
        RecentActivity {}
    )
}

#[component]
fn RecentActivity() -> Element {
    rsx!(
        div {
            class: "card bg-base-200",
            div {
                class: "card-body",
                h2 {
                    class: "card-title mb-4",
                    "Upcoming Timers"
                }
                div {
                    class: "text-center py-8 opacity-50",
                    "No upcoming timers"
                }
            }
        }
    )
}
