mod modals;
mod table;

use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{
            page::{ErrorPage, LoadingPage},
            Page,
        },
        model::error::ApiError,
        route::admin::server::{ActionTabs, GuildInfoHeader, PingFormatsCache, ServerAdminTab},
        router::Route,
    },
    model::discord::DiscordGuildDto,
};

use modals::CreatePingFormatModal;
use table::{PingFormatPagination, PingFormatsTable};

#[cfg(feature = "web")]
use crate::client::api::{discord_guild::get_discord_guild_by_id, ping_format::get_ping_formats};

#[component]
pub fn ServerAdminPingFormat(guild_id: u64) -> Element {
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
        Title { "Ping Formats | Black Rose Timerboard" }
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
                    ActionTabs { guild_id, active_tab: ServerAdminTab::PingFormats }
                    div {
                        class: "space-y-6",
                        PingFormatsSection { guild_id }
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
fn PingFormatsSection(guild_id: u64) -> Element {
    let mut cache = use_context::<Signal<PingFormatsCache>>();
    let mut error = use_signal(|| None::<ApiError>);
    let mut show_create_modal = use_signal(|| false);

    // Get page and per_page from cache
    let page = use_signal(|| cache.read().page);
    let per_page = use_signal(|| cache.read().per_page);
    let refetch_trigger = use_signal(|| 0u32);

    // Fetch ping formats - resource automatically re-runs when page(), per_page(), or refetch_trigger changes
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        let _ = refetch_trigger();
        get_ping_formats(guild_id, page(), per_page()).await
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
                    tracing::error!("Failed to fetch ping formats: {}", err);
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
                        "Ping Formats"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create_modal.set(true),
                        "Add Ping Format"
                    }
                }

                // Content
                if let Some(data) = cache.read().data.clone() {
                    if data.ping_formats.is_empty() {
                        div {
                            class: "text-center py-8 opacity-50",
                            "No ping formats configured"
                        }
                    } else {
                        PingFormatsTable {
                            data: data.clone(),
                            guild_id,
                            cache,
                            refetch_trigger
                        }
                        PingFormatPagination {
                            page,
                            per_page,
                            pagination_data: data.clone(),
                            cache
                        }
                    }
                } else if let Some(err) = error() {
                    div {
                        class: "alert alert-error",
                        span { "Error loading ping formats: {err.message}" }
                    }
                } else {
                    div {
                        class: "text-center py-8",
                        span { class: "loading loading-spinner loading-lg" }
                    }
                }

                // Create Ping Format Modal
                CreatePingFormatModal {
                    guild_id,
                    show: show_create_modal,
                    refetch_trigger
                }
            }
        }
    )
}
