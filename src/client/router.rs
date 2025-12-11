use dioxus::prelude::*;

use crate::client::component::{Layout, ProtectedLayout};
use crate::client::route::{timerboard::Home, Login};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/login")]
    Login {},

    #[layout(ProtectedLayout)]
    #[route("/")]
    Home {},
}
