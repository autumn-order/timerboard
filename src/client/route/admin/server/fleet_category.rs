use dioxus::prelude::*;

use crate::client::{
    component::{
        page::{ErrorPage, LoadingPage},
        Page,
    },
    model::error::ApiError,
    router::Route,
};
use crate::model::discord::DiscordGuildDto;

use super::{ActionTabs, GuildInfoHeader, ServerAdminTab};

#[component]
pub fn ServerAdminFleetCategory(guild_id: u64) -> Element {
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
                guild.set(Some(Err(err.clone())));
                fetched.set(true);
            }
            None => (),
        }
    }

    rsx! {
        Title { "Fleet Categories | Black Rose Timerboard" }
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
                    GuildInfoHeader { guild_data: guild_data.clone() }
                    ActionTabs { guild_id, active_tab: ServerAdminTab::FleetCategories }
                    div {
                        class: "space-y-6",
                        FleetCategoriesSection { guild_id }
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
fn FleetCategoriesSection(guild_id: u64) -> Element {
    rsx!(
        div {
            class: "card bg-base-200",
            div {
                class: "card-body",
                div {
                    class: "flex justify-between items-center mb-4",
                    h2 {
                        class: "card-title",
                        "Fleet Categories"
                    }
                    button {
                        class: "btn btn-primary",
                        "Add Category"
                    }
                }
                div {
                    class: "text-center py-8 opacity-50",
                    "No fleet categories configured"
                }
            }
        }
    )
}
