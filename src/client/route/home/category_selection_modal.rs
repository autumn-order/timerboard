use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::client::component::Modal;

use super::ManageableCategoriesCache;

#[cfg(feature = "web")]
use crate::client::api::user::get_user_manageable_categories;

/// Modal for selecting which fleet category to create a fleet in
#[component]
pub fn CategorySelectionModal(
    guild_id: u64,
    mut show: Signal<bool>,
    on_category_selected: EventHandler<i32>,
) -> Element {
    let mut cache = use_context::<Signal<ManageableCategoriesCache>>();

    // Fetch categories only if not cached or guild changed
    #[cfg(feature = "web")]
    {
        let mut should_fetch = use_signal(|| false);

        // Check cache and initiate fetch if needed
        use_effect(use_reactive!(|guild_id| {
            // Skip if already fetching
            if should_fetch() {
                return;
            }

            let mut cache_state = cache.write();

            // Check if we need to fetch
            let needs_fetch = (cache_state.guild_id != Some(guild_id)
                || cache_state.data.is_none())
                && !cache_state.is_fetching;

            if needs_fetch {
                // Set fetching flag while we still hold the lock
                cache_state.is_fetching = true;
                drop(cache_state);
                should_fetch.set(true);
            }
        }));

        let future = use_resource(move || async move {
            if should_fetch() {
                Some(get_user_manageable_categories(guild_id).await)
            } else {
                None
            }
        });

        use_effect(move || {
            if let Some(Some(result)) = future.read_unchecked().as_ref() {
                match result {
                    Ok(categories) => {
                        cache.write().guild_id = Some(guild_id);
                        cache.write().data = Some(Ok(categories.clone()));
                        cache.write().is_fetching = false;
                        should_fetch.set(false);
                    }
                    Err(err) => {
                        tracing::error!("Failed to fetch categories: {}", err);
                        cache.write().guild_id = Some(guild_id);
                        cache.write().data = Some(Err(err.clone()));
                        cache.write().is_fetching = false;
                        should_fetch.set(false);
                    }
                }
            }
        });
    }

    let categories = cache
        .read()
        .data
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();

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
