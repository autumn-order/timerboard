use dioxus::prelude::*;

use crate::client::component::Page;

#[component]
pub fn Home() -> Element {
    rsx! {
        Title { "Black Rose Timerboard" }
        Page {
            class: "flex items-center justify-center w-full h-full",
            p {
                "This is a homepage"
            }
        }
    }
}
