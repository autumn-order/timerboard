use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_brands_icons::FaDiscord, Icon};

use crate::client::{
    constant::SITE_NAME,
    model::{
        auth::AuthState,
        cache::{Cache, CacheState},
    },
    router::Route,
};

const LOGO: Asset = asset!(
    "/assets/logo.webp",
    AssetOptions::image().with_size(ImageSize::Manual {
        width: 48,
        height: 48
    })
);

#[component]
pub fn Header() -> Element {
    let auth_cache = use_context::<Cache<AuthState>>();
    let cache = auth_cache.read();

    rsx!(div {
        class: "fixed flex justify-between gap-4 w-full h-20 py-2 px-4 bg-base-200 z-20",
        div {
            class: "flex items-center",
            div {
                Link {
                    to: Route::Home {},
                    div {
                        class: "flex items-center gap-3",
                        img {
                            src: LOGO,
                        }
                        p {
                            class: "md:text-xl text-wrap",
                            {SITE_NAME}
                        }
                    }
                }
            }

        }
        div {
            class: "flex items-center gap-2",
            {render_auth_buttons(&cache)}
        }
    })
}

fn render_auth_buttons(cache: &CacheState<AuthState>) -> Element {
    match cache {
        CacheState::Fetched(AuthState::Authenticated(user)) => rsx! {
            if user.admin {
                Link {
                    to: Route::AdminServers {},
                    class: "btn btn-outline",
                    "Admin"
                }
            }
            a {
                href: "/api/auth/logout",
                div {
                    class: "btn btn-outline",
                    "Logout"
                }
            }
        },
        CacheState::Fetched(AuthState::NotLoggedIn) | CacheState::Error(_) => rsx! {
            a {
                href: "/api/auth/login",
                div {
                    class: "btn btn-outline flex gap-2 items-center",
                    Icon {
                        width: 22,
                        height: 22,
                        icon: FaDiscord
                    }
                    "Login"
                }
            }
        },
        CacheState::NotFetched => rsx! {},
    }
}
