use dioxus::prelude::*;

use crate::client::{router::Route, store::user::UserState};

#[component]
pub fn Header() -> Element {
    let user_store = use_context::<Store<UserState>>();

    let user_logged_in = user_store.read().user.is_some();
    let fetch_completed = user_store.read().fetched;

    rsx!(div {
        class: "fixed flex justify-between w-full h-20 py-2 px-4 bg-base-200",
        div {
            class: "flex items-center",
            div {
                Link {
                    to: Route::Home {},
                    p {
                        "Timerboard"
                    }
                }
            }

        }
        div {
            class: "flex items-center",
            if fetch_completed && user_logged_in {
                div {
                    class: "btn btn-outline",
                    a {
                        href: "/api/auth/logout",
                        p {
                            "Logout"
                        }
                    }
                }
            } else if fetch_completed {
                div {
                    class: "btn btn-outline",
                    a {
                        href: "/api/auth/login",
                        p {
                            "Login"
                        }
                    }
                }
            }
        }
    })
}
