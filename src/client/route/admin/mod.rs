pub mod server;

pub use server::{ServerAdmin, ServerAdminFleetCategory, ServerAdminPingFormat};

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
use crate::model::discord::DiscordGuildDto;

#[cfg(feature = "web")]
use crate::client::api::discord_guild::get_all_discord_guilds;

#[component]
pub fn Admin() -> Element {
    let mut guilds = use_signal(|| None::<Result<Vec<DiscordGuildDto>, ApiError>>);
    let mut fetched = use_signal(|| false);

    // Fetch guilds on first load
    #[cfg(feature = "web")]
    {
        let future = use_resource(|| async move { get_all_discord_guilds().await });

        match &*future.read_unchecked() {
            Some(Ok(guild_list)) => {
                guilds.set(Some(Ok(guild_list.clone())));
                fetched.set(true);
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch guilds: {}", err);
                guilds.set(Some(Err(err.clone())));
                fetched.set(true);
            }
            None => (),
        }
    }

    rsx! {
        Title { "Admin | Black Rose Timerboard" }
        if let Some(Ok(guild_list)) = guilds() {
            Page {
                class: "flex flex-col items-center w-full h-full",
                div {
                    class: "flex items-center justify-between gap-4 w-full max-w-6xl mb-6",
                    h1 {
                        class: "text-lg sm:text-2xl",
                        "Manage Servers"
                    }
                    a {
                        href: "/api/admin/bot/add",
                        class: "btn btn-primary",
                        "Add New Server"
                    }
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 w-full max-w-6xl",
                    for guild in guild_list {
                        Link {
                            to: Route::ServerAdmin { guild_id: guild.guild_id as u64 },
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
        } else if let Some(Err(error)) = guilds() {
            ErrorPage { status: error.status, message: error.message }
        } else {
            LoadingPage { }
        }
    }
}
