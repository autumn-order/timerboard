use dioxus::prelude::*;

use crate::client::{constant::SITE_NAME, model::auth::AuthContext, router::Route};

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
    let mut auth_context = use_context_provider(AuthContext::new);

    // Fetch user on first load
    #[cfg(feature = "web")]
    {
        auth_context.fetch_user();
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
