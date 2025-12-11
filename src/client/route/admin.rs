use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::client::component::Page;
use crate::model::{api::ErrorDto, discord::DiscordGuildDto};

#[cfg(feature = "web")]
pub async fn get_all_discord_guilds() -> Result<Vec<DiscordGuildDto>, String> {
    use reqwasm::http::Request;

    let response = Request::get("/api/admin/discord/guilds")
        .credentials(reqwasm::http::RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    match response.status() {
        200 => {
            let guilds = response
                .json::<Vec<DiscordGuildDto>>()
                .await
                .map_err(|e| format!("Failed to parse Discord guild data: {}", e))?;
            Ok(guilds)
        }
        _ => {
            if let Ok(error_dto) = response.json::<ErrorDto>().await {
                Err(format!(
                    "Request failed with status {}: {}",
                    response.status(),
                    error_dto.error
                ))
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!(
                    "Request failed with status {}: {}",
                    response.status(),
                    error_text
                ))
            }
        }
    }
}

#[component]
pub fn Admin() -> Element {
    let mut guilds = use_signal(|| None::<Result<Vec<DiscordGuildDto>, String>>);
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
                tracing::error!(err);
                guilds.set(Some(Err(err.clone())));
                fetched.set(true);
            }
            None => (),
        }
    }

    rsx! {
        Title { "Admin | Black Rose Timerboard" }
        Page {
            class: "flex flex-col items-center w-full h-full",
            h1 {
                class: "text-2xl font-bold mb-6",
                "Admin - Discord Servers"
            }
            if let Some(Ok(guild_list)) = guilds() {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 w-full max-w-6xl",
                    for guild in guild_list {
                        div {
                            class: "flex items-center gap-4 p-4 border border-neutral rounded-lg",
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
                                    class: "text-sm",
                                    "ID: {guild.guild_id}"
                                }
                            }
                        }
                    }
                }
            } else if let Some(Err(error)) = guilds() {
                div {
                    class: "text-red-500",
                    p { "Error loading guilds: {error}" }
                }
            } else {
                p { "Loading guilds..." }
            }
        }
    }
}
