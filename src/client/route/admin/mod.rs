pub mod timerboard;

pub use timerboard::TimerboardAdmin;

use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::client::{
    component::{
        page::{ErrorPage, LoadingPage},
        Page,
    },
    model::error::ApiError,
};
use crate::model::{api::ErrorDto, discord::DiscordGuildDto};

#[cfg(feature = "web")]
pub async fn get_all_discord_guilds() -> Result<Vec<DiscordGuildDto>, ApiError> {
    use reqwasm::http::Request;

    let response = Request::get("/api/admin/discord/guilds")
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
            let guilds = response
                .json::<Vec<DiscordGuildDto>>()
                .await
                .map_err(|e| ApiError {
                    status: 500,
                    message: format!("Failed to parse Discord guild data: {}", e),
                })?;
            Ok(guilds)
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
pub async fn get_discord_guild_by_id(guild_id: u64) -> Result<DiscordGuildDto, ApiError> {
    use reqwasm::http::Request;

    let response = Request::get(&format!("/api/admin/discord/guilds/{}", guild_id))
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
            let guild = response
                .json::<DiscordGuildDto>()
                .await
                .map_err(|e| ApiError {
                    status: 500,
                    message: format!("Failed to parse Discord guild data: {}", e),
                })?;
            Ok(guild)
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
                            to: "/admin/{guild.guild_id}",
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
