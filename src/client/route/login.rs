use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_brands_icons::FaDiscord, Icon};

use crate::client::{
    component::{page::LoadingPage, Page},
    constant::SITE_NAME,
    model::auth::{AuthContext, AuthState},
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
    let auth_context = use_context::<AuthContext>();
    let nav = navigator();

    // Handle redirect for authenticated users
    {
        let auth_context = use_context::<AuthContext>();
        use_effect(move || {
            let state = auth_context.read();
            if matches!(&*state, AuthState::Authenticated(_)) {
                nav.push(Route::Home {});
            }
        });
    }

    let state = auth_context.read();

    rsx! {
        Title { "Login | {SITE_NAME}" }
        match &*state {
            AuthState::Initializing => rsx! {
                LoadingPage {}
            },
            AuthState::Authenticated(_) => rsx! {
                // Render nothing while redirecting
                LoadingPage {}
            },
            AuthState::NotLoggedIn | AuthState::Error(_) => rsx! {
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
