use dioxus::prelude::*;

use crate::client::component::Page;

#[component]
pub fn Admin() -> Element {
    rsx! {
        Title { "Admin | Black Rose Timerboard" }
        Page {
            class: "flex items-center justify-center w-full h-full",
            p {
                "This is the admin page"
            }
        }
    }
}
