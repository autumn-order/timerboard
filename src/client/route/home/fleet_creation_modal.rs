use chrono::{Datelike, Timelike, Utc};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use std::collections::HashMap;

use crate::{
    client::{
        component::{
            searchable_dropdown::{DropdownItem, SearchableDropdown},
            FullScreenModal, UtcDateTimeInput,
        },
        model::error::ApiError,
        store::user::UserState,
    },
    model::{category::FleetCategoryListItemDto, fleet::CreateFleetDto},
};

#[cfg(feature = "web")]
use crate::client::api::{
    fleet::{create_fleet, get_category_details, get_guild_members},
    user::get_user_manageable_categories,
};

/// Modal for creating a new fleet with all required details
#[component]
pub fn FleetCreationModal(guild_id: u64, category_id: i32, mut show: Signal<bool>) -> Element {
    let user_store = use_context::<Store<UserState>>();
    let current_user = user_store.read().user.clone();
    let current_user_id = current_user.as_ref().map(|user| user.discord_id);

    // Track selected category (can be changed via dropdown)
    let mut selected_category_id = use_signal(move || category_id);

    // Update selected_category_id when category_id prop changes (e.g., reopening modal with different category)
    use_effect(use_reactive!(|category_id| {
        selected_category_id.set(category_id);
    }));

    let mut manageable_categories =
        use_signal(|| None::<Result<Vec<FleetCategoryListItemDto>, ApiError>>);
    let mut category_details =
        use_signal(|| None::<Result<crate::model::category::FleetCategoryDetailsDto, ApiError>>);
    let mut guild_members =
        use_signal(|| None::<Result<Vec<crate::model::discord::DiscordGuildMemberDto>, ApiError>>);

    // Fleet form state
    let mut fleet_name = use_signal(|| String::new());

    // Pre-fill fleet datetime with current UTC time in format "YYYY-MM-DD HH:MM"
    let current_datetime = {
        let now = Utc::now();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute()
        )
    };
    let fleet_datetime = use_signal(move || current_datetime.clone());

    // Pre-fill fleet commander with current user's discord_id
    let mut fleet_commander_id = use_signal(move || current_user_id);

    let mut fleet_description = use_signal(|| String::new());
    let mut field_values = use_signal(|| std::collections::HashMap::<i32, String>::new());

    // Searchable dropdown state
    let mut commander_search = use_signal(|| String::new());
    let mut show_commander_dropdown = use_signal(|| false);

    // Submission state
    let mut is_submitting = use_signal(|| false);
    let mut submission_error = use_signal(|| None::<String>);

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
                    fleet_description.set(String::new());
                    field_values.set(HashMap::new());
                    is_submitting.set(false);
                    show.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to create fleet: {}", err);
                    submission_error.set(Some(format!("Failed to create fleet: {}", err)));
                    is_submitting.set(false);
                }
            }
        }
    });

    // Fetch manageable categories
    #[cfg(feature = "web")]
    {
        let future =
            use_resource(move || async move { get_user_manageable_categories(guild_id).await });

        match &*future.read_unchecked() {
            Some(Ok(categories)) => {
                manageable_categories.set(Some(Ok(categories.clone())));
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch categories: {}", err);
                manageable_categories.set(Some(Err(err.clone())));
            }
            None => (),
        }
    }

    // Fetch category details (re-fetches when selected_category_id changes)
    #[cfg(feature = "web")]
    {
        let future = use_resource(use_reactive!(|selected_category_id| async move {
            get_category_details(guild_id, selected_category_id()).await
        }));

        match &*future.read_unchecked() {
            Some(Ok(details)) => {
                category_details.set(Some(Ok(details.clone())));
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch category details: {}", err);
                category_details.set(Some(Err(err.clone())));
            }
            None => (),
        }
    }

    // Fetch guild members
    #[cfg(feature = "web")]
    {
        let future = use_resource(move || async move { get_guild_members(guild_id).await });

        match &*future.read_unchecked() {
            Some(Ok(members)) => {
                guild_members.set(Some(Ok(members.clone())));
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch guild members: {}", err);
                guild_members.set(Some(Err(err.clone())));
            }
            None => (),
        }
    }

    rsx! {
        FullScreenModal {
            show,
            title: "Create Fleet",
            prevent_close: false,
            div {
                class: "space-y-4 overflow-y-auto max-h-[calc(100vh-200px)] sm:max-h-[calc(90vh-200px)]",

                if let Some(Ok(details)) = category_details() {
                    // Essentials Section
                    div {
                        class: "space-y-4",

                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                            // Category (dropdown to switch categories)
                            div {
                                class: "flex flex-col gap-2",
                                label {
                                    class: "label",
                                    span { class: "label-text", "Category" }
                                }
                                {
                                    if let Some(Ok(categories)) = manageable_categories() {
                                        rsx! {
                                            select {
                                                class: "select select-bordered w-full",
                                                value: "{selected_category_id()}",
                                                onchange: move |e| {
                                                    if let Ok(new_id) = e.value().parse::<i32>() {
                                                        selected_category_id.set(new_id);
                                                        // Clear field values when category changes
                                                        field_values.set(std::collections::HashMap::new());
                                                    }
                                                },
                                                for category in categories {
                                                    option {
                                                        key: "{category.id}",
                                                        value: "{category.id}",
                                                        selected: category.id == selected_category_id(),
                                                        "{category.name}"
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        rsx! {
                                            input {
                                                r#type: "text",
                                                class: "input input-bordered w-full",
                                                value: "{details.name}",
                                                disabled: true
                                            }
                                        }
                                    }
                                }
                            }

                            // Fleet Name
                            div {
                                class: "flex flex-col gap-2",
                                label {
                                    class: "label",
                                    span { class: "label-text", "Fleet Name" }
                                }
                                input {
                                    r#type: "text",
                                    class: "input input-bordered w-full",
                                    placeholder: "Enter fleet name...",
                                    value: "{fleet_name}",
                                    oninput: move |e| fleet_name.set(e.value())
                                }
                            }

                            // Fleet DateTime (UTC, 24-hour format) - Pre-filled with current datetime
                            div {
                                class: "flex flex-col gap-2",
                                label {
                                    class: "label",
                                    span { class: "label-text", "Fleet Date & Time" }
                                }
                                UtcDateTimeInput {
                                    value: fleet_datetime,
                                    required: true,
                                }
                            }

                            // Fleet Commander - Searchable Dropdown (Pre-filled with current user)
                            div {
                                class: "flex flex-col gap-2",
                                label {
                                    class: "label",
                                    span { class: "label-text", "Fleet Commander" }
                                }

                                if let Some(Ok(members)) = guild_members() {
                                    {
                                        let selected_member = members.iter().find(|m| Some(m.user_id) == fleet_commander_id());
                                        let display_value = selected_member.map(|m| format!("{} (@{})", m.display_name, m.username));

                                        let search_lower = commander_search().to_lowercase();
                                        let mut filtered_members: Vec<_> = members.iter()
                                            .filter(|m| {
                                                search_lower.is_empty() ||
                                                m.display_name.to_lowercase().contains(&search_lower) ||
                                                m.username.to_lowercase().contains(&search_lower)
                                            })
                                            .collect();

                                        // Sort to always put the logged-in user at the top
                                        if let Some(current_id) = current_user_id {
                                            filtered_members.sort_by_key(|m| {
                                                if m.user_id == current_id {
                                                    0 // Current user first
                                                } else {
                                                    1 // Everyone else after
                                                }
                                            });
                                        }

                                        rsx! {
                                            SearchableDropdown {
                                                search_query: commander_search,
                                                placeholder: "Search for a fleet commander...".to_string(),
                                                display_value,
                                                required: true,
                                                has_items: !filtered_members.is_empty(),
                                                show_dropdown_signal: show_commander_dropdown,
                                                empty_message: "No guild members found".to_string(),
                                                not_found_message: "No matching members found".to_string(),

                                                for member in filtered_members {
                                                    {
                                                        let member_id = member.user_id;
                                                        let member_display = member.display_name.clone();
                                                        let member_username = member.username.clone();
                                                        let is_selected = Some(member_id) == fleet_commander_id();

                                                        rsx! {
                                                            DropdownItem {
                                                                key: "{member_id}",
                                                                selected: is_selected,
                                                                on_select: move |_| {
                                                                    fleet_commander_id.set(Some(member_id));
                                                                    show_commander_dropdown.set(false);
                                                                    commander_search.set(String::new());
                                                                },
                                                                div {
                                                                    class: "flex flex-col",
                                                                    span { class: "font-semibold", "{member_display}" }
                                                                    span { class: "text-sm opacity-70", "@{member_username}" }
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
                    }

                    // Ping Format Fields Section
                    if !details.fields.is_empty() {
                        div {
                            class: "space-y-4",

                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                                for field in details.fields.clone() {
                                    {
                                        let field_id = field.id;
                                        let field_name = field.name.clone();
                                        rsx! {
                                            div {
                                                key: "{field_id}",
                                                class: "flex flex-col gap-2",
                                                label {
                                                    class: "label",
                                                    span { class: "label-text", "{field_name}" }
                                                }
                                                input {
                                                    r#type: "text",
                                                    class: "input input-bordered w-full",
                                                    placeholder: "Enter {field_name.to_lowercase()}...",
                                                    value: "{field_values().get(&field_id).cloned().unwrap_or_default()}",
                                                    oninput: move |e| {
                                                        let mut values = field_values();
                                                        values.insert(field_id, e.value());
                                                        field_values.set(values);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Fleet Description Section
                    div {
                        class: "space-y-2",
                        h3 {
                            class: "text-lg font-bold",
                            "Description"
                        }
                        textarea {
                            class: "textarea textarea-bordered h-32 w-full",
                            placeholder: "Enter fleet description...",
                            value: "{fleet_description}",
                            oninput: move |e| fleet_description.set(e.value())
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
                            disabled: fleet_name().is_empty() || fleet_datetime().is_empty() || fleet_commander_id().is_none() || is_submitting(),
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
                } else if let Some(Err(_)) = category_details() {
                    div {
                        class: "text-center py-8",
                        p {
                            class: "text-error",
                            "Failed to load category details"
                        }
                    }
                } else {
                    div {
                        class: "text-center py-8",
                        span {
                            class: "loading loading-spinner loading-lg"
                        }
                    }
                }
            }
        }
    }
}
