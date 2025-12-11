use dioxus::prelude::*;

use crate::client::{component::Header, router::Route};

#[component]
pub fn Layout() -> Element {
    rsx!(div {
        Header {  }
        Outlet::<Route> {}
    })
}
