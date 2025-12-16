mod category_selection_modal;
mod create_fleet_button;
mod fleet_modals;
mod fleet_table;

pub use category_selection_modal::CategorySelectionModal;
pub use create_fleet_button::CreateFleetButton;
pub use fleet_modals::{FleetCreationModal, FleetViewEditModal, ViewEditMode};
pub use fleet_table::FleetTable;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use std::collections::HashMap;

use crate::{
    client::{
        component::page::{ErrorPage, LoadingPage, Page},
        constant::SITE_NAME,
        model::error::ApiError,
    },
    model::{
        category::{FleetCategoryDetailsDto, FleetCategoryListItemDto},
        discord::{DiscordGuildDto, DiscordGuildMemberDto},
    },
};

// Cache for manageable categories per guild
#[derive(Clone, Default)]
pub struct ManageableCategoriesCache {
    pub guild_id: Option<u64>,
    pub data: Option<Result<Vec<FleetCategoryListItemDto>, ApiError>>,
    pub is_fetching: bool,
}

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
}

#[cfg(feature = "web")]
use crate::client::api::user::get_user_guilds;

#[component]
pub fn Home() -> Element {
    let mut guilds = use_signal(|| None::<Result<Vec<DiscordGuildDto>, ApiError>>);
    let mut selected_guild_id = use_signal(|| None::<u64>);
    let mut show_guild_dropdown = use_signal(|| false);
    let mut show_create_modal = use_signal(|| false);
    let mut show_fleet_creation = use_signal(|| false);
    let mut selected_category_id = use_signal(|| None::<i32>);
    let mut refetch_trigger = use_signal(|| 0u32);

    // Provide caches for child components
    let mut manageable_categories_cache =
        use_context_provider(|| Signal::new(ManageableCategoriesCache::default()));
    let mut guild_members_cache =
        use_context_provider(|| Signal::new(GuildMembersCache::default()));
    let mut category_details_cache =
        use_context_provider(|| Signal::new(CategoryDetailsCache::default()));

    // Fetch user's guilds on first load
    #[cfg(feature = "web")]
    {
        let future = use_resource(|| async move { get_user_guilds().await });

        match &*future.read_unchecked() {
            Some(Ok(guild_list)) => {
                guilds.set(Some(Ok(guild_list.clone())));

                // Auto-select first guild (lowest ID) if nothing selected yet
                if selected_guild_id().is_none() && !guild_list.is_empty() {
                    // Find guild with lowest guild_id
                    let first_guild = guild_list.iter().min_by_key(|g| g.guild_id);

                    if let Some(guild) = first_guild {
                        selected_guild_id.set(Some(guild.guild_id as u64));
                    }
                }
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch guilds: {}", err);
                guilds.set(Some(Err(err.clone())));
            }
            None => (),
        }
    }

    // Get selected guild name for display
    let selected_guild = guilds().and_then(|result| {
        result.ok().and_then(|guild_list| {
            selected_guild_id()
                .and_then(|id| guild_list.into_iter().find(|g| g.guild_id as u64 == id))
        })
    });

    rsx! {
        Title { "{SITE_NAME}" }
        if let Some(Ok(guild_list)) = guilds() {
            if guild_list.is_empty() {
                // No guilds available
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
            } else {
                // Has guilds
                Page {
                    class: "flex flex-col items-center w-full h-full",
                    div {
                        class: "w-full max-w-6xl px-4 py-6",

                        // Server selector header
                        div {
                            class: "mb-6",
                            div {
                                class: "flex flex-wrap items-center justify-between gap-4",

                                // Clickable guild header with dropdown
                                div {
                                    class: "relative",
                                    if let Some(guild) = selected_guild.clone() {
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
                                    }

                                    // Guild dropdown menu
                                    if show_guild_dropdown() {
                                        div {
                                            class: "absolute top-full left-0 mt-2 w-80 bg-base-100 rounded-box shadow-lg border border-base-300 z-50",
                                            div {
                                                class: "p-2",
                                                div {
                                                    class: "max-h-96 overflow-y-auto",
                                                    for guild in guild_list.clone() {
                                                        {
                                                            let guild_id = guild.guild_id as u64;
                                                            let is_selected = selected_guild_id() == Some(guild_id);
                                                            rsx! {
                                                                button {
                                                                    key: "{guild_id}",
                                                                    class: "w-full flex items-center gap-3 p-3 rounded-box hover:bg-base-200 transition-colors",
                                                                    class: if is_selected { "bg-base-200" },
                                                                    onclick: move |_| {
                                                                        selected_guild_id.set(Some(guild_id));
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
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Create Fleet Button
                                div {
                                    class: "w-full sm:w-auto",
                                    if let Some(guild_id) = selected_guild_id() {
                                        CreateFleetButton {
                                            guild_id,
                                            show_create_modal
                                        }
                                    }
                                }
                            }
                        }

                        // Fleet Timerboard
                        div {
                            if let Some(guild_id) = selected_guild_id() {
                                FleetTable {
                                    guild_id,
                                    refetch_trigger
                                }
                            }
                        }
                    }
                }

                // Category Selection Modal
                if let Some(guild_id) = selected_guild_id() {
                    CategorySelectionModal {
                        guild_id,
                        show: show_create_modal,
                        on_category_selected: move |category_id| {
                            selected_category_id.set(Some(category_id));
                            show_create_modal.set(false);
                            show_fleet_creation.set(true);
                        }
                    }
                }

                // Fleet Creation Modal
                if let Some(guild_id) = selected_guild_id() {
                    if let Some(category_id) = selected_category_id() {
                        FleetCreationModal {
                            guild_id,
                            category_id,
                            show: show_fleet_creation,
                            on_success: move |_| {
                                refetch_trigger.set(refetch_trigger() + 1);
                            }
                        }
                    }
                }
            }
        } else if let Some(Err(error)) = guilds() {
            ErrorPage { status: error.status, message: error.message }
        } else {
            LoadingPage { }
        }

        // Click outside to close dropdown
        if show_guild_dropdown() {
            div {
                class: "fixed inset-0 z-40",
                onclick: move |_| show_guild_dropdown.set(false),
            }
        }
    }
}
