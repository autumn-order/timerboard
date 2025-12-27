use dioxus::prelude::*;

use crate::client::{
    constant::SITE_NAME,
    model::{auth::AuthState, cache::Cache},
    router::Route,
};

#[cfg(feature = "web")]
use crate::client::api::user::get_user;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const LOGO: Asset = asset!(
    "/assets/logo.webp",
    AssetOptions::image().with_size(ImageSize::Manual {
        width: 256,
        height: 256
    })
);

#[component]
pub fn App() -> Element {
    let mut auth_cache = use_context_provider(Cache::<AuthState>::new);

    // Fetch user on first load
    #[cfg(feature = "web")]
    {
        auth_cache.fetch(|| async { get_user().await.map(AuthState::from) });
    }

    rsx! {
        Title { "{SITE_NAME}" }
        document::Link { rel: "icon", href: FAVICON }
        document::Meta {
            name: "og:image",
            content: LOGO
        }
        document::Meta {
            name: "twitter:image",
            content: LOGO
        }
        document::Meta {
            name: "description",
            content: " Discord-based fleet timerboard for EVE Online "
        }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}
