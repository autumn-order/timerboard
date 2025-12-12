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

#[component]
pub fn TimerboardAdmin(guild_id: u64) -> Element {
    let mut guild = use_signal(|| None::<Result<DiscordGuildDto, ApiError>>);
    let mut fetched = use_signal(|| false);

    // Fetch guild on first load
    #[cfg(feature = "web")]
    {
        use crate::client::route::admin::get_discord_guild_by_id;

        let future = use_resource(move || async move { get_discord_guild_by_id(guild_id).await });

        match &*future.read_unchecked() {
            Some(Ok(guild_data)) => {
                guild.set(Some(Ok(guild_data.clone())));
                fetched.set(true);
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch guild: {}", err);
                guild.set(Some(Err(err.clone())));
                fetched.set(true);
            }
            None => (),
        }
    }

    rsx! {
        Title { "Timerboard Admin | Black Rose Timerboard" }
        if let Some(Ok(guild_data)) = guild() {
            Page {
                class: "flex flex-col items-center w-full h-full",
                div {
                    class: "w-full max-w-6xl",
                    Link {
                        to: Route::Admin {},
                        class: "btn btn-ghost mb-4",
                        "← Back to Servers"
                    }
                    GuildInfoHeader { guild_data: guild_data }
                    ActionTabs {  }
                    div {
                        class: "space-y-6",
                        OverviewSection { }
                    }
                }
            }
        } else if let Some(Err(error)) = guild() {
            ErrorPage { status: error.status, message: error.message }
        } else {
            LoadingPage { }
        }
    }
}

#[component]
fn GuildInfoHeader(guild_data: DiscordGuildDto) -> Element {
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
fn ActionTabs() -> Element {
    rsx! (
        div {
            role: "tablist",
            class: "tabs tabs-bordered mb-6",
            a {
                role: "tab",
                class: "tab tab-active",
                "Overview"
            }
            a {
                role: "tab",
                class: "tab",
                "Fleet Categories"
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
