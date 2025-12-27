use dioxus::prelude::*;

use crate::{
    client::{constant::SITE_NAME, model::Cache, router::Route},
    model::user::UserDto,
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
    let mut user_cache = use_context_provider(Cache::<UserDto>::new);

    // Fetch user on first load
    #[cfg(feature = "web")]
    {
        let future = use_resource(|| async move { get_user().await });

        if let Some(result) = &*future.read_unchecked() {
            let _ = user_cache
                .write()
                .update_from_optional_result(result.clone());
        }
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
