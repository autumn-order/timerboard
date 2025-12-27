use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_brands_icons::FaDiscord, Icon};

use crate::client::{
    component::{page::LoadingPage, Page},
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
        width: 176,
        height: 176
    })
);

#[component]
pub fn Login() -> Element {
    let auth_cache = use_context::<Cache<AuthState>>();
    let nav = navigator();

    // Handle redirect for authenticated users
    {
        let auth_cache = auth_cache.clone();
        use_effect(move || {
            let cache = auth_cache.read();
            if matches!(&*cache, CacheState::Fetched(AuthState::Authenticated(_))) {
                nav.push(Route::Home {});
            }
        });
    }

    let cache = auth_cache.read();

    rsx! {
        Title { "Login | {SITE_NAME}" }
        match &*cache {
            CacheState::NotFetched => rsx! {
                LoadingPage {}
            },
            CacheState::Fetched(AuthState::Authenticated(_)) => rsx! {
                // Render nothing while redirecting
                LoadingPage {}
            },
            CacheState::Fetched(AuthState::NotLoggedIn) | CacheState::Error(_) => rsx! {
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
    }
}
