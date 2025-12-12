use dioxus::prelude::*;

use crate::client::component::{Layout, RequiresAdmin, RequiresLoggedIn};
use crate::client::route::{admin::TimerboardAdmin, Admin, Home, Login, NotFound};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/login")]
    Login {},

    #[layout(RequiresLoggedIn)]
    #[route("/")]
    Home {},

    #[end_layout]

    #[layout(RequiresAdmin)]
    #[nest("/admin")]
        #[route("/")]
        Admin {},

        #[route("/:guild_id")]
        TimerboardAdmin { guild_id: u64 },
    #[end_nest]
    #[end_layout]

    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
