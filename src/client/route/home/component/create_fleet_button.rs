use dioxus::prelude::*;

use crate::{client::model::cache::Cache, model::category::FleetCategoryListItemDto};

#[component]
pub fn CreateFleetButton(guild_id: u64, mut show_create_modal: Signal<bool>) -> Element {
    let cache = use_context::<Cache<Vec<FleetCategoryListItemDto>>>();
    let manageable_categories = cache.read();

    let can_create = manageable_categories
        .data()
        .map(|categories| !categories.is_empty())
        .unwrap_or(false);

    rsx! {
        if can_create {
            button {
                class: "btn btn-primary w-full sm:w-auto",
                onclick: move |_| show_create_modal.set(true),
                "Create Fleet"
            }
        }
    }
}
