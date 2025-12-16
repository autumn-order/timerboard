pub mod fleet_category;
pub mod ping_format;

pub use fleet_category::ServerAdminFleetCategory;
pub use ping_format::ServerAdminPingFormat;

use dioxus::prelude::*;

use crate::{client::router::Route, model::discord::DiscordGuildDto};

use crate::model::{category::PaginatedFleetCategoriesDto, ping_format::PaginatedPingFormatsDto};

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
