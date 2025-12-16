pub mod server;
pub mod users;

pub use server::{ServerAdminFleetCategory, ServerAdminPingFormat};
pub use users::AdminUsers;

use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::client::{
    component::{
        page::{ErrorPage, LoadingPage},
        Page,
    },
    constant::SITE_NAME,
    model::error::ApiError,
    router::Route,
};
use crate::model::{discord::DiscordGuildDto, user::UserDto};

#[cfg(feature = "web")]
use crate::client::api::discord_guild::get_all_discord_guilds;

/// Cached guilds data
#[derive(Clone, PartialEq)]
pub struct GuildsCache {
    pub data: Option<Vec<DiscordGuildDto>>,
}

/// Cached admin users data
#[derive(Clone, PartialEq)]
pub struct AdminUsersCache {
    pub data: Option<Vec<UserDto>>,
}

/// Layout component that provides context for admin pages
/// This layout is automatically dropped when navigating away from admin pages,
/// which cleans up the context.
#[component]
pub fn AdminLayout() -> Element {
    // Initialize the guilds cache - persists across tab navigation within admin
    use_context_provider(|| Signal::new(GuildsCache { data: None }));

    // Initialize the admin users cache - persists across tab navigation within admin
    use_context_provider(|| Signal::new(AdminUsersCache { data: None }));

    rsx! {
        // Render child routes (AdminServers or AdminUsers)
        Outlet::<Route> {}
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AdminTab {
    Servers,
    Users,
}

#[component]
pub fn AdminServers() -> Element {
    let mut cache = use_context::<Signal<GuildsCache>>();
    let mut error = use_signal(|| None::<ApiError>);

    // Fetch guilds if not already cached
    #[cfg(feature = "web")]
    {
        let needs_fetch = cache.read().data.is_none();

        if needs_fetch {
            let future = use_resource(|| async move { get_all_discord_guilds().await });

            match &*future.read_unchecked() {
                Some(Ok(guild_list)) => {
                    cache.write().data = Some(guild_list.clone());
                    error.set(None);
                }
                Some(Err(err)) => {
                    tracing::error!("Failed to fetch guilds: {}", err);
                    cache.write().data = None;
                    error.set(Some(err.clone()));
                }
                None => (),
            }
        }
    }

    rsx! {
        Title { "Admin - Servers | {SITE_NAME}" }
        if let Some(guild_list) = cache.read().data.clone() {
            Page {
                class: "flex flex-col items-center w-full h-full",
                div {
                    class: "w-full max-w-6xl",
                    h1 {
                        class: "text-lg sm:text-2xl mb-6",
                        "Admin Panel"
                    }

                    // Tabs
                    AdminTabs { active_tab: AdminTab::Servers }

                    // Content
                    div {
                        class: "flex items-center justify-between gap-4 mb-6",
                        h2 {
                            class: "text-lg font-semibold",
                            "Manage Servers"
                        }
                        a {
                            href: "/api/admin/bot/add",
                            class: "btn btn-primary",
                            "Add New Server"
                        }
                    }
                    div {
                        class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                        for guild in guild_list {
                            Link {
                                to: Route::ServerAdminFleetCategory { guild_id: guild.guild_id as u64 },
                                class: "card bg-base-200 hover:bg-base-300 transition-colors",
                                div {
                                    class: "card-body",
                                    div {
                                        class: "flex items-center gap-4",
                                        if let Some(icon_hash) = &guild.icon_hash {
                                            img {
                                                src: "https://cdn.discordapp.com/icons/{guild.guild_id}/{icon_hash}.png",
                                                alt: "{guild.name} icon",
                                                class: "w-12 h-12 rounded-full",
                                            }
                                        } else {
                                            div {
                                                class: "w-12 h-12 rounded-full bg-neutral flex items-center justify-center font-bold",
                                                "{guild.name.chars().next().unwrap_or('?')}"
                                            }
                                        }
                                        div {
                                            class: "flex-1",
                                            h3 {
                                                class: "font-semibold",
                                                "{guild.name}"
                                            }
                                            p {
                                                class: "text-sm opacity-70",
                                                "ID: {guild.guild_id}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
pub fn AdminTabs(active_tab: AdminTab) -> Element {
    rsx! (
        div {
            role: "tablist",
            class: "tabs tabs-bordered mb-6",
            Link {
                to: Route::AdminServers {},
                role: "tab",
                class: if active_tab == AdminTab::Servers { "tab tab-active" } else { "tab" },
                "Servers"
            }
            Link {
                to: Route::AdminUsers {},
                role: "tab",
                class: if active_tab == AdminTab::Users { "tab tab-active" } else { "tab" },
                "Users"
            }
        }
    )
}
