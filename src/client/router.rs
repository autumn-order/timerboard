use dioxus::prelude::*;

use crate::client::component::{Layout, RequiresAdmin, RequiresLoggedIn};
use crate::client::route::{Admin, Home, Login, NotFound};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/login")]
    Login {},

    #[route("/:..segments")]
    NotFound { segments: Vec<String> },

    #[layout(RequiresLoggedIn)]
    #[route("/")]
    Home {},

    #[end_layout]

    #[layout(RequiresAdmin)]
    #[route("/admin")]
    Admin {},
}
