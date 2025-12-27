use chrono::{Datelike, Timelike, Utc};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use std::collections::HashMap;

use crate::{
    client::{component::modal::FullScreenModal, model::error::ApiError, store::user::UserState},
    model::{fleet::CreateFleetDto, ping_format::PingFormatFieldType},
};

use super::FleetFormFields;
use crate::client::route::home::{
    CategoryDetailsCache, GuildMembersCache, ManageableCategoriesCache,
};

#[cfg(feature = "web")]
use crate::client::api::{
    fleet::{create_fleet, get_category_details, get_guild_members},
    user::get_user_manageable_categories,
};

/// Modal for creating a new fleet with all required details
#[component]
pub fn FleetCreationModal(
    guild_id: u64,
    category_id: i32,
    mut show: Signal<bool>,
    on_success: EventHandler<()>,
) -> Element {
    let user_store = use_context::<Store<UserState>>();
    let current_user = user_store.read().user.clone();
    let current_user_id = current_user.as_ref().map(|user| user.discord_id);

    // Track selected category (can be changed via dropdown)
    let mut selected_category_id = use_signal(move || category_id);

    // Update selected_category_id when category_id prop changes (e.g., reopening modal with different category)
    use_effect(use_reactive!(|category_id| {
        selected_category_id.set(category_id);
    }));

    // Fleet form state
    let mut fleet_name = use_signal(String::new);

    // Pre-fill fleet datetime with next 5-minute interval to give users buffer time
    // This prevents "time in the past" errors if they take a few minutes to fill out the form
    let current_datetime = {
        let now = Utc::now();
        let current_minute = now.minute();
        // Round up to next 5-minute mark (e.g., 14:32 -> 14:35, 14:35 -> 14:35)
        let rounded_minute = current_minute.div_ceil(5) * 5;

        let rounded_time = if rounded_minute >= 60 {
            // Roll over to next hour
            now + chrono::Duration::minutes((60 - current_minute) as i64)
        } else {
            now + chrono::Duration::minutes((rounded_minute - current_minute) as i64)
        };

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            rounded_time.year(),
            rounded_time.month(),
            rounded_time.day(),
            rounded_time.hour(),
            rounded_time.minute()
        )
    };
    let mut fleet_datetime = use_signal(move || current_datetime.clone());

    // Pre-fill fleet commander with current user's discord_id
    let mut fleet_commander_id = use_signal(move || current_user_id);

    let mut fleet_description = use_signal(String::new);
    let mut field_values = use_signal(std::collections::HashMap::<i32, String>::new);

    // Fleet visibility options
    let mut hidden = use_signal(|| false);
    let mut disable_reminder = use_signal(|| false);

    // Submission state
    let mut is_submitting = use_signal(|| false);
    let mut submission_error = use_signal(|| None::<String>);

    // Datetime validation error
    let datetime_error = use_signal(|| None::<String>);

    // Handle fleet creation submission
    #[cfg(feature = "web")]
    let create_future = use_resource(move || async move {
        if is_submitting() {
            let dto = CreateFleetDto {
                category_id: selected_category_id(),
                name: fleet_name(),
                commander_id: fleet_commander_id().unwrap_or(0),
                fleet_time: fleet_datetime(),
                description: if fleet_description().is_empty() {
                    None
                } else {
                    Some(fleet_description())
                },
                field_values: field_values(),
                hidden: hidden(),
                disable_reminder: disable_reminder(),
            };
            Some(create_fleet(guild_id, dto).await)
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = create_future.read_unchecked().as_ref() {
            match result {
                Ok(_fleet) => {
                    tracing::info!("Fleet created successfully");
                    // Reset form and close modal
                    fleet_name.set(String::new());

                    // Reset datetime to next 5-minute interval
                    let now = Utc::now();
                    let current_minute = now.minute();
                    let rounded_minute = current_minute.div_ceil(5) * 5;
                    let rounded_time = if rounded_minute >= 60 {
                        now + chrono::Duration::minutes((60 - current_minute) as i64)
                    } else {
                        now + chrono::Duration::minutes((rounded_minute - current_minute) as i64)
                    };
                    fleet_datetime.set(format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}",
                        rounded_time.year(),
                        rounded_time.month(),
                        rounded_time.day(),
                        rounded_time.hour(),
                        rounded_time.minute()
                    ));

                    fleet_commander_id.set(current_user_id);
                    fleet_description.set(String::new());
                    field_values.set(HashMap::new());
                    hidden.set(false);
                    disable_reminder.set(false);
                    submission_error.set(None);
                    is_submitting.set(false);
                    show.set(false);
                    // Notify parent to refetch fleets
                    on_success.call(());
                }
                Err(err) => {
                    tracing::error!("Failed to create fleet: {}", err);
                    submission_error.set(Some(format!("Failed to create fleet: {}", err)));
                    is_submitting.set(false);
                }
            }
        }
    });

    // Use manageable categories cache from context
    let mut manageable_categories_cache = use_context::<Signal<ManageableCategoriesCache>>();
    let mut should_fetch_categories = use_signal(|| false);

    // Fetch manageable categories only if not cached or guild changed
    #[cfg(feature = "web")]
    {
        // Check cache and initiate fetch if needed
        use_effect(use_reactive!(|guild_id| {
            // Skip if already fetching
            if should_fetch_categories() {
                return;
            }

            let mut cache_state = manageable_categories_cache.write();

            // Check if we need to fetch
            let needs_fetch = (cache_state.guild_id != Some(guild_id)
                || cache_state.data.is_none())
                && !cache_state.is_fetching;

            if needs_fetch {
                // Set fetching flag while we still hold the lock
                cache_state.is_fetching = true;
                drop(cache_state);
                should_fetch_categories.set(true);
            }
        }));

        let manageable_categories_resource = use_resource(move || async move {
            if should_fetch_categories() {
                Some(get_user_manageable_categories(guild_id).await)
            } else {
                None
            }
        });

        use_effect(move || {
            if let Some(Some(result)) = manageable_categories_resource.read().as_ref() {
                manageable_categories_cache.write().guild_id = Some(guild_id);
                manageable_categories_cache.write().data = Some(result.clone());
                manageable_categories_cache.write().is_fetching = false;
                should_fetch_categories.set(false);
            }
        });
    }

    let manageable_categories = manageable_categories_cache.read().data.clone();

    // Use category details cache from context
    let mut category_details_cache = use_context::<Signal<CategoryDetailsCache>>();

    let mut category_details =
        use_signal(|| None::<Result<crate::model::category::FleetCategoryDetailsDto, ApiError>>);

    // Fetch category details only if not cached
    #[cfg(feature = "web")]
    {
        let mut should_fetch_details = use_signal(|| false);

        // Check cache and update local state from cache
        use_effect(use_reactive!(|selected_category_id| {
            let current_category_id = selected_category_id();

            // First, check if we have cached data
            let cached_details = category_details_cache
                .read()
                .data
                .get(&current_category_id)
                .cloned();

            if let Some(cached) = cached_details {
                // Use cached data
                if category_details().is_none()
                    || category_details()
                        .as_ref()
                        .map(|d| d.as_ref().ok().map(|dto| dto.id))
                        != Some(Some(current_category_id))
                {
                    category_details.set(Some(cached));
                }
                return;
            }

            // Skip if already fetching this category
            if should_fetch_details() {
                return;
            }

            let mut cache_state = category_details_cache.write();

            // Check if another component is already fetching this category
            if cache_state.is_fetching.contains(&current_category_id) {
                return;
            }

            // Check again if data arrived while we were waiting for write lock
            if cache_state.data.contains_key(&current_category_id) {
                return;
            }

            // Claim this fetch
            cache_state.is_fetching.insert(current_category_id);
            drop(cache_state);
            should_fetch_details.set(true);
        }));

        let category_details_resource = use_resource(move || async move {
            if should_fetch_details() {
                let current_category_id = selected_category_id();
                Some(get_category_details(guild_id, current_category_id).await)
            } else {
                None
            }
        });

        use_effect(move || {
            if let Some(Some(result)) = category_details_resource.read().as_ref() {
                let current_category_id = selected_category_id();
                category_details.set(Some(result.clone()));

                let mut cache_state = category_details_cache.write();
                cache_state.data.insert(current_category_id, result.clone());
                cache_state.is_fetching.remove(&current_category_id);
                drop(cache_state);

                should_fetch_details.set(false);
            }
        });
    }

    // Reset form when modal opens/closes
    use_effect(use_reactive!(|show| {
        if show() {
            // Reset form fields when modal opens
            fleet_name.set(String::new());
            fleet_description.set(String::new());
            field_values.set(HashMap::new());
            hidden.set(false);
            disable_reminder.set(false);
        }
    }));

    // Pre-fill field values with defaults when category details change
    use_effect(use_reactive!(|category_details| {
        if let Some(Ok(details)) = category_details() {
            let mut defaults = HashMap::new();
            for field in &details.fields {
                match field.field_type {
                    PingFormatFieldType::Bool => {
                        // Initialize bool fields to "false"
                        defaults.insert(field.id, "false".to_string());
                    }
                    PingFormatFieldType::Text => {
                        // Initialize text fields with first default value if available
                        if !field.default_field_values.is_empty() {
                            defaults.insert(field.id, field.default_field_values[0].clone());
                        }
                    }
                }
            }
            // Always set field_values to defaults (empty map if no defaults)
            field_values.set(defaults);
        }
    }));

    // Use guild members cache from context
    let mut guild_members_cache = use_context::<Signal<GuildMembersCache>>();
    let mut should_fetch_members = use_signal(|| false);

    // Fetch guild members only if not cached or guild changed
    #[cfg(feature = "web")]
    {
        // Check cache and initiate fetch if needed
        use_effect(use_reactive!(|guild_id| {
            // Skip if already fetching
            if should_fetch_members() {
                return;
            }

            let mut cache_state = guild_members_cache.write();

            // Check if we need to fetch
            let needs_fetch = (cache_state.guild_id != Some(guild_id)
                || cache_state.data.is_none())
                && !cache_state.is_fetching;

            if needs_fetch {
                // Set fetching flag while we still hold the lock
                cache_state.is_fetching = true;
                drop(cache_state);
                should_fetch_members.set(true);
            }
        }));

        let guild_members_resource = use_resource(move || async move {
            if should_fetch_members() {
                Some(get_guild_members(guild_id).await)
            } else {
                None
            }
        });

        use_effect(move || {
            if let Some(Some(result)) = guild_members_resource.read().as_ref() {
                guild_members_cache.write().guild_id = Some(guild_id);
                guild_members_cache.write().data = Some(result.clone());
                guild_members_cache.write().is_fetching = false;
                should_fetch_members.set(false);
            }
        });
    }

    let guild_members = guild_members_cache.read().data.clone();

    rsx! {
        FullScreenModal {
            show,
            title: "Create Fleet",
            prevent_close: false,
            div {
                class: "space-y-4 overflow-y-auto max-h-[calc(100vh-200px)] sm:max-h-[calc(90vh-200px)]",

                // Use shared form fields component
                FleetFormFields {
                    guild_id,
                    fleet_name,
                    fleet_datetime,
                    fleet_commander_id,
                    fleet_description,
                    field_values,
                    category_details,
                    guild_members: use_signal(move || guild_members.clone()),
                    is_submitting: is_submitting(),
                    current_user_id,
                    hidden,
                    disable_reminder,
                    selected_category_id: Some(selected_category_id),
                    manageable_categories: Some(use_signal(move || manageable_categories.clone())),
                    datetime_error_signal: Some(datetime_error),
                }
                // Submission error
                // Submission error message
                if let Some(error) = submission_error() {
                    div {
                        class: "alert alert-error mt-4",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            class: "stroke-current shrink-0 h-6 w-6",
                            fill: "none",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                            }
                        }
                        span { "{error}" }
                    }
                }

                // Action Buttons
                div {
                    class: "flex gap-2 justify-end pt-4",
                    button {
                        class: "btn",
                        onclick: move |_| show.set(false),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: fleet_name().is_empty() || fleet_datetime().is_empty() || fleet_commander_id().is_none() || is_submitting() || datetime_error().is_some(),
                        onclick: move |_| {
                            is_submitting.set(true);
                            submission_error.set(None);
                        },
                        if is_submitting() {
                            span { class: "loading loading-spinner loading-sm" }
                            " Creating..."
                        } else {
                            "Create Fleet"
                        }
                    }
                }
            }
        }
    }
}
