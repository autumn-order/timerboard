use dioxus::prelude::*;

use crate::{
    client::{component::Modal, model::cache::GuildCache},
    model::category::FleetCategoryListItemDto,
};

/// Modal for selecting which fleet category to create a fleet in
#[component]
pub fn CategorySelectionModal(
    guild_id: u64,
    mut show: Signal<bool>,
    on_category_selected: EventHandler<i32>,
) -> Element {
    let manageable_categories_cache =
        use_context::<Signal<GuildCache<Vec<FleetCategoryListItemDto>>>>();

    let categories = manageable_categories_cache()
        .data()
        .cloned()
        .unwrap_or(Vec::new());

    rsx! {
        Modal {
            show,
            title: "Select Fleet Category",
            prevent_close: false,
            div {
                class: "space-y-4",
                if categories.is_empty() {
                    div {
                        class: "text-center py-8",
                        p {
                            class: "text-base-content/70",
                            "No categories available for fleet creation."
                        }
                    }
                } else {
                    div {
                        class: "grid grid-cols-1 gap-3 max-h-96 overflow-y-auto",
                        for category in categories {
                            {
                                let category_id = category.id;
                                let category_name = category.name.clone();

                                rsx! {
                                    button {
                                        key: "{category_id}",
                                        class: "block w-full text-left p-4 rounded-box border border-base-300 hover:bg-base-200 hover:border-primary transition-all",
                                        onclick: move |_| {
                                            on_category_selected.call(category_id);
                                        },
                                        div {
                                            class: "font-medium text-lg",
                                            "{category_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
