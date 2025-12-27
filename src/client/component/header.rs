use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_brands_icons::FaDiscord, Icon};

use crate::{
    client::{constant::SITE_NAME, model::Cache, router::Route},
    model::user::UserDto,
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
    let user_cache = use_context::<Cache<UserDto>>();

    let user_logged_in = user_cache.read().data.is_some();
    let user_is_admin = user_cache.read().data.as_ref().is_some_and(|u| u.admin);
    let fetch_completed = user_cache.read().fetched;

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
            if fetch_completed && user_logged_in {
                if user_is_admin {
                    Link {
                        to: Route::AdminServers {},
                        class: "btn btn-outline",
                        p {
                            "Admin"
                        }
                    }
                }
                a {
                    href: "/api/auth/logout",
                    div {
                        class: "btn btn-outline",
                        p {
                            "Logout"
                        }
                    }
                }
            } else if fetch_completed {
                a {
                    href: "/api/auth/login",
                    div {
                        class: "btn btn-outline flex gap-2 items-center",
                        Icon {
                            width: 22,
                            height: 22,
                            icon: FaDiscord
                        }
                        p {
                            "Login"
                        }
                    }
                }
            }
        }
    })
}
