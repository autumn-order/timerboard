mod component;

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;

use crate::{
    client::{
        component::page::{ErrorPage, LoadingPage, Page},
        constant::SITE_NAME,
        model::{
            cache::{Cache, CacheState},
            error::ApiError,
        },
        route::home::component::{
            CategorySelectionModal, CreateFleetButton, FleetCreationModal, FleetTable,
        },
    },
    model::{
        category::{FleetCategoryDetailsDto, FleetCategoryListItemDto},
        discord::{DiscordGuildDto, DiscordGuildMemberDto},
    },
};

// Cache for guild members per guild
#[derive(Clone, Default)]
pub struct GuildMembersCache {
    pub guild_id: Option<u64>,
    pub data: Option<Result<Vec<DiscordGuildMemberDto>, ApiError>>,
    pub is_fetching: bool,
}

// Cache for category details per category
#[derive(Clone, Default)]
pub struct CategoryDetailsCache {
    pub data: HashMap<i32, Result<FleetCategoryDetailsDto, ApiError>>,
    pub is_fetching: HashSet<i32>, // Track which category IDs are currently being fetched
}

#[cfg(feature = "web")]
use crate::client::api::user::{get_user_guilds, get_user_manageable_categories};

#[component]
pub fn Home() -> Element {
    // Provide caches for child components
    let mut manageable_categories_cache =
        use_context_provider(Cache::<Vec<FleetCategoryListItemDto>>::new);
    let _guild_members_cache = use_context_provider(|| Signal::new(GuildMembersCache::default()));
    let _category_details_cache =
        use_context_provider(|| Signal::new(CategoryDetailsCache::default()));

    let mut guilds_cache = Cache::<Vec<DiscordGuildDto>>::new();
    let mut selected_guild = use_signal(|| None::<DiscordGuildDto>);
    let refetch_trigger = use_signal(|| 0u32);

    #[cfg(feature = "web")]
    {
        // Fetch user's guilds on first load
        guilds_cache.fetch(get_user_guilds);

        // Fetch user's manageable categories for guild when selected guild changes
        use_effect(move || {
            if let Some(guild) = selected_guild() {
                let guild_id = guild.guild_id;
                manageable_categories_cache
                    .refetch(move || get_user_manageable_categories(guild_id))
            }
        });
    }

    let guilds = guilds_cache.read();
    if let CacheState::Fetched(guilds_list) = &*guilds {
        if selected_guild().is_none() && !guilds_list.is_empty() {
            let first_guild = guilds_list.iter().min_by_key(|g| g.guild_id);

            if let Some(guild) = first_guild {
                selected_guild.set(Some(guild.clone()));
            }
        }
    }

    rsx! {
        Title { "{SITE_NAME}" }
        match &*guilds {
            CacheState::NotFetched | CacheState::Loading => rsx! {
                LoadingPage { }
            },
            CacheState::Fetched(guilds_list) if guilds_list.is_empty() => rsx! {
                Page {
                    class: "flex items-center justify-center w-full h-full",
                    div {
                        h2 {
                            class: "card-title justify-center text-xl mb-4",
                            "No Timerboards Available"
                        }
                        p {
                            class: "mb-4",
                            "You don't have access to any timerboards."
                        }
                    }
                }
            },
            CacheState::Fetched(guilds_list) => rsx! {
                Page {
                    class: "flex flex-col items-center w-full h-full",
                    div {
                        class: "w-full max-w-6xl px-4 py-6",
                        TimerboardHeader {
                            selected_guild,
                            guilds_list: guilds_list.clone(),
                            refetch_trigger,
                        }

                        if let Some(guild) = selected_guild() {
                            FleetTable {
                                guild_id: guild.guild_id,
                                refetch_trigger
                            }
                        }
                    }
                }
            },
            CacheState::Error(error) => rsx! {
                ErrorPage { status: error.status, message: error.message.to_string() }
            }
        }
    }
}

#[component]
fn TimerboardHeader(
    selected_guild: Signal<Option<DiscordGuildDto>>,
    guilds_list: Vec<DiscordGuildDto>,
    refetch_trigger: Signal<u32>,
) -> Element {
    let mut show_create_modal = use_signal(|| false);
    let mut show_fleet_creation = use_signal(|| false);
    let mut selected_category_id = use_signal(|| None::<i32>);

    rsx!(
        div {
            class: "mb-6",
            div {
                class: "flex flex-wrap items-center justify-between gap-4",
                if let Some(guild) = selected_guild() {
                    {
                        let guild_id = guild.guild_id;

                        rsx! {
                            ServerSelector {
                                selected_guild,
                                guild,
                                guilds_list
                            }

                            CreateFleetButton {
                                guild_id: guild_id,
                                show_create_modal
                            }
                        }
                    }
                }
            }
        }

        // Category Selection Modal
        if let Some(guild) = selected_guild() {
            CategorySelectionModal {
                guild_id: guild.guild_id,
                show: show_create_modal,
                on_category_selected: move |category_id| {
                    selected_category_id.set(Some(category_id));
                    show_create_modal.set(false);
                    show_fleet_creation.set(true);
                }
            }
        }

        // Fleet Creation Modal
        if let Some(guild) = selected_guild() {
            if let Some(category_id) = selected_category_id() {
                FleetCreationModal {
                    guild_id: guild.guild_id,
                    category_id,
                    show: show_fleet_creation,
                    on_success: move |_| {
                        refetch_trigger.set(refetch_trigger() + 1);
                    }
                }
            }
        }


    )
}

#[component]
fn ServerSelector(
    selected_guild: Signal<Option<DiscordGuildDto>>,
    guild: DiscordGuildDto,
    guilds_list: Vec<DiscordGuildDto>,
) -> Element {
    let mut show_guild_dropdown = use_signal(|| false);

    rsx! {
        div {
            class: "relative",
            button {
                class: "flex items-center gap-3 hover:opacity-80 transition-opacity",
                onclick: move |_| show_guild_dropdown.set(!show_guild_dropdown()),
                if let Some(icon_hash) = &guild.icon_hash {
                    img {
                        src: "https://cdn.discordapp.com/icons/{guild.guild_id}/{icon_hash}.png",
                        alt: "{guild.name} icon",
                        class: "w-10 h-10 rounded-full",
                    }
                } else {
                    div {
                        class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center font-bold",
                        "{guild.name.chars().next().unwrap_or('?')}"
                    }
                }
                h1 {
                    class: "text-xl font-bold",
                    "{guild.name}"
                }
                // Chevron icon
                svg {
                    class: "w-5 h-5 transition-transform",
                    class: if show_guild_dropdown() { "rotate-180" },
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M19 9l-7 7-7-7"
                    }
                }
            }

            // Guild dropdown menu
            if show_guild_dropdown() {
                GuildDropdown {
                    selected_guild,
                    show_guild_dropdown,
                    guilds_list
                }
            }
        }
    }
}

#[component]
fn GuildDropdown(
    selected_guild: Signal<Option<DiscordGuildDto>>,
    show_guild_dropdown: Signal<bool>,
    guilds_list: Vec<DiscordGuildDto>,
) -> Element {
    rsx! {
        div {
            class: "absolute top-full left-0 mt-2 w-80 bg-base-100 rounded-box shadow-lg border border-base-300 z-50",
            div {
                class: "max-h-96 p-2 overflow-y-auto",
                for guild in guilds_list {
                    GuildButton {
                        selected_guild,
                        guild,
                        show_guild_dropdown
                    }
                }
            }

            // Click outside to close dropdown
            div {
                class: "fixed inset-0 z-40",
                onclick: move |_| show_guild_dropdown.set(false),
            }
        }
    }
}

#[component]
fn GuildButton(
    selected_guild: Signal<Option<DiscordGuildDto>>,
    guild: DiscordGuildDto,
    show_guild_dropdown: Signal<bool>,
) -> Element {
    let guild_id = guild.guild_id;
    let is_selected = selected_guild().as_ref().map(|g| g.guild_id) == Some(guild_id);

    rsx! {
        button {
            key: "{guild_id}",
            class: "w-full flex items-center gap-3 p-3 rounded-box hover:bg-base-200 transition-colors",
            class: if is_selected { "bg-base-200" },
            onclick: move |_| {
                selected_guild.set(Some(guild.clone()));
                show_guild_dropdown.set(false);
            },
            if let Some(icon) = guild.icon_hash.as_ref() {
                img {
                    src: "https://cdn.discordapp.com/icons/{guild_id}/{icon}.png",
                    alt: "{guild.name} icon",
                    class: "w-10 h-10 rounded-full",
                }
            } else {
                div {
                    class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center font-bold",
                    "{guild.name.chars().next().unwrap_or('?')}"
                }
            }
            div {
                class: "flex-1 text-left",
                div {
                    class: "font-medium",
                    "{guild.name}"
                }
            }
        }
    }
}
