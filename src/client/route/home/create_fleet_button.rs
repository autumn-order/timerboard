use dioxus::prelude::*;
use dioxus_logger::tracing;

use super::ManageableCategoriesCache;

#[cfg(feature = "web")]
use crate::client::api::user::get_user_manageable_categories;

#[component]
pub fn CreateFleetButton(guild_id: u64, mut show_create_modal: Signal<bool>) -> Element {
    // Use shared cache
    let mut cache = use_context::<Signal<ManageableCategoriesCache>>();

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

    let can_create = cache
        .read()
        .data
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|categories| !categories.is_empty())
        .unwrap_or(false);

    rsx! {
        if can_create {
            button {
                class: "btn btn-primary w-full",
                onclick: move |_| show_create_modal.set(true),
                "Create Fleet"
            }
        }
    }
}
