use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_brands_icons::FaDiscord, Icon};

use crate::client::{component::Page, constant::SITE_NAME, router::Route, store::user::UserState};

const LOGO: Asset = asset!(
    "/assets/logo.webp",
    AssetOptions::image().with_size(ImageSize::Manual {
        width: 176,
        height: 176
    })
);

#[component]
pub fn Login() -> Element {
    let user_store = use_context::<Store<UserState>>();
    let nav = navigator();

    let user_logged_in = user_store.read().user.is_some();
    let fetch_completed = user_store.read().fetched;

    // Redirect authenticed user to home after fetch completes
    use_effect(use_reactive!(|(user_logged_in, fetch_completed)| {
        if user_logged_in && fetch_completed {
            nav.push(Route::Home {});
        }
    }));

    rsx! {
        Title { "Login | {SITE_NAME}" }
        Page {
            class: "flex flex-col gap-6 items-center justify-center w-full h-full",
            div {
                class: "flex flex-col items-center gap-4",
                img {
                    width: 176,
                    height: 176,
                    src: LOGO,
                }
                p {
                    class: "text-2xl",
                    {SITE_NAME}
                }
            }
            div {
                a {
                    href: "/api/auth/login",
                    div {
                        class: "btn btn-outline flex gap-2 items-center",
                        Icon {
                            width: 24,
                            height: 24,
                            icon: FaDiscord
                        }
                        p {
                            "Login with Discord"
                        }
                    }
                }
            }
        }
    }
}
