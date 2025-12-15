use chrono::{Datelike, Timelike, Utc};
use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{
            page::{ErrorPage, LoadingPage},
            searchable_dropdown::{DropdownItem, SearchableDropdown},
            FullScreenModal, Modal, Page, UtcDateTimeInput,
        },
        model::error::ApiError,
        store::user::UserState,
    },
    model::{category::FleetCategoryListItemDto, discord::DiscordGuildDto},
};

#[cfg(feature = "web")]
use crate::client::api::{
    fleet::{get_category_details, get_guild_members},
    user::{get_user_guilds, get_user_manageable_categories},
};

#[component]
pub fn Home() -> Element {
    let mut guilds = use_signal(|| None::<Result<Vec<DiscordGuildDto>, ApiError>>);
    let mut selected_guild_id = use_signal(|| None::<u64>);
    let mut show_guild_dropdown = use_signal(|| false);
    let mut show_create_modal = use_signal(|| false);
    let mut show_fleet_creation = use_signal(|| false);
    let mut selected_category_id = use_signal(|| None::<i32>);

    // Fetch user's guilds on first load
    #[cfg(feature = "web")]
    {
        let future = use_resource(|| async move { get_user_guilds().await });

        match &*future.read_unchecked() {
            Some(Ok(guild_list)) => {
                guilds.set(Some(Ok(guild_list.clone())));

                // Auto-select first guild (lowest ID) if nothing selected yet
                if selected_guild_id().is_none() && !guild_list.is_empty() {
                    // Find guild with lowest guild_id
                    let first_guild = guild_list.iter().min_by_key(|g| g.guild_id);

                    if let Some(guild) = first_guild {
                        selected_guild_id.set(Some(guild.guild_id as u64));
                    }
                }
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch guilds: {}", err);
                guilds.set(Some(Err(err.clone())));
            }
            None => (),
        }
    }

    // Get selected guild name for display
    let selected_guild = guilds().and_then(|result| {
        result.ok().and_then(|guild_list| {
            selected_guild_id()
                .and_then(|id| guild_list.into_iter().find(|g| g.guild_id as u64 == id))
        })
    });

    rsx! {
        Title { "Black Rose Timerboard" }
        if let Some(Ok(guild_list)) = guilds() {
            if guild_list.is_empty() {
                // No guilds available
                Page {
                    class: "flex items-center justify-center w-full h-full",
                    div {
                        h2 {
                            class: "card-title justify-center text-xl mb-4",
                            "No Timerboards Available"
                        }
                        p {
                            class: "mb-4",
                            "You don't have access to any timerboards."
                        }
                    }
                }
            } else {
                // Has guilds
                Page {
                    class: "flex flex-col items-center w-full h-full",
                    div {
                        class: "w-full max-w-6xl px-4 py-6",

                        // Server selector header
                        div {
                            class: "mb-6",
                            div {
                                class: "flex flex-wrap items-center justify-between gap-4",

                                // Clickable guild header with dropdown
                                div {
                                    class: "relative",
                                    if let Some(guild) = selected_guild.clone() {
                                        button {
                                            class: "flex items-center gap-3 hover:opacity-80 transition-opacity",
                                            onclick: move |_| show_guild_dropdown.set(!show_guild_dropdown()),
                                            if let Some(icon_hash) = &guild.icon_hash {
                                                img {
                                                    src: "https://cdn.discordapp.com/icons/{guild.guild_id}/{icon_hash}.png",
                                                    alt: "{guild.name} icon",
                                                    class: "w-10 h-10 rounded-full",
                                                }
                                            } else {
                                                div {
                                                    class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center font-bold",
                                                    "{guild.name.chars().next().unwrap_or('?')}"
                                                }
                                            }
                                            h1 {
                                                class: "text-xl font-bold",
                                                "{guild.name}"
                                            }
                                            // Chevron icon
                                            svg {
                                                class: "w-5 h-5 transition-transform",
                                                class: if show_guild_dropdown() { "rotate-180" },
                                                xmlns: "http://www.w3.org/2000/svg",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                stroke: "currentColor",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    stroke_width: "2",
                                                    d: "M19 9l-7 7-7-7"
                                                }
                                            }
                                        }
                                    }

                                    // Guild dropdown menu
                                    if show_guild_dropdown() {
                                        div {
                                            class: "absolute top-full left-0 mt-2 w-80 bg-base-100 rounded-box shadow-lg border border-base-300 z-50",
                                            div {
                                                class: "p-2",
                                                div {
                                                    class: "max-h-96 overflow-y-auto",
                                                    for guild in guild_list.clone() {
                                                        {
                                                            let guild_id = guild.guild_id as u64;
                                                            let is_selected = selected_guild_id() == Some(guild_id);
                                                            rsx! {
                                                                button {
                                                                    key: "{guild_id}",
                                                                    class: "w-full flex items-center gap-3 p-3 rounded-box hover:bg-base-200 transition-colors",
                                                                    class: if is_selected { "bg-base-200" },
                                                                    onclick: move |_| {
                                                                        selected_guild_id.set(Some(guild_id));
                                                                        show_guild_dropdown.set(false);
                                                                    },
                                                                    if let Some(icon) = guild.icon_hash.as_ref() {
                                                                        img {
                                                                            src: "https://cdn.discordapp.com/icons/{guild_id}/{icon}.png",
                                                                            alt: "{guild.name} icon",
                                                                            class: "w-10 h-10 rounded-full",
                                                                        }
                                                                    } else {
                                                                        div {
                                                                            class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center font-bold",
                                                                            "{guild.name.chars().next().unwrap_or('?')}"
                                                                        }
                                                                    }
                                                                    div {
                                                                        class: "flex-1 text-left",
                                                                        div {
                                                                            class: "font-medium",
                                                                            "{guild.name}"
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

                                // Create Fleet Button
                                div {
                                    class: "w-full sm:w-auto",
                                    if let Some(guild_id) = selected_guild_id() {
                                        CreateFleetButton {
                                            guild_id,
                                            show_create_modal
                                        }
                                    }
                                }
                            }
                        }

                        // Timerboard content (placeholder)
                        div {
                            class: "flex items-center justify-center min-h-[400px]",
                            if let Some(guild) = selected_guild.clone() {
                                div {
                                    class: "text-center",
                                    h2 {
                                        class: "text-3xl font-bold opacity-50",
                                        "{guild.name}"
                                    }
                                    p {
                                        class: "text-sm opacity-30 mt-2",
                                        "Timerboard content coming soon"
                                    }
                                }
                            } else {
                                p {
                                    class: "opacity-50",
                                    "Select a server to view its timerboard"
                                }
                            }
                        }
                    }
                }

                // Create Fleet Modal (Category Selection)
                if let Some(guild_id) = selected_guild_id() {
                    CreateFleetModal {
                        guild_id,
                        show: show_create_modal,
                        on_category_selected: move |category_id| {
                            selected_category_id.set(Some(category_id));
                            show_create_modal.set(false);
                            show_fleet_creation.set(true);
                        }
                    }
                }

                // Fleet Creation Modal
                if let Some(guild_id) = selected_guild_id() {
                    if let Some(category_id) = selected_category_id() {
                        FleetCreationModal {
                            guild_id,
                            category_id,
                            show: show_fleet_creation
                        }
                    }
                }
            }
        } else if let Some(Err(error)) = guilds() {
            ErrorPage { status: error.status, message: error.message }
        } else {
            LoadingPage { }
        }

        // Click outside to close dropdown
        if show_guild_dropdown() {
            div {
                class: "fixed inset-0 z-40",
                onclick: move |_| show_guild_dropdown.set(false),
            }
        }
    }
}

#[component]
fn CreateFleetButton(guild_id: u64, mut show_create_modal: Signal<bool>) -> Element {
    let mut manageable_categories =
        use_signal(|| None::<Result<Vec<FleetCategoryListItemDto>, ApiError>>);

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

    let can_create = manageable_categories()
        .and_then(|result| result.ok())
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

#[component]
fn CreateFleetModal(
    guild_id: u64,
    mut show: Signal<bool>,
    on_category_selected: EventHandler<i32>,
) -> Element {
    let mut manageable_categories =
        use_signal(|| None::<Result<Vec<FleetCategoryListItemDto>, ApiError>>);

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

    let categories = manageable_categories()
        .and_then(|result| result.ok())
        .unwrap_or_default();

    rsx! {
        Modal {
            show,
            title: "Create Fleet",
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
                                        div {
                                            class: "text-sm text-base-content/70 mt-1",
                                            "Create a new fleet in this category"
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

#[component]
fn FleetCreationModal(guild_id: u64, category_id: i32, mut show: Signal<bool>) -> Element {
    let user_store = use_context::<Store<UserState>>();
    let current_user = user_store.read().user.clone();

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
    let mut fleet_commander_id =
        use_signal(move || current_user.as_ref().map(|user| user.discord_id));

    let mut fleet_description = use_signal(|| String::new());
    let mut field_values = use_signal(|| std::collections::HashMap::<i32, String>::new());

    // Searchable dropdown state
    let mut commander_search = use_signal(|| String::new());
    let mut show_commander_dropdown = use_signal(|| false);

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
                                        let filtered_members: Vec<_> = members.iter()
                                            .filter(|m| {
                                                search_lower.is_empty() ||
                                                m.display_name.to_lowercase().contains(&search_lower) ||
                                                m.username.to_lowercase().contains(&search_lower)
                                            })
                                            .collect();

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
                            disabled: fleet_name().is_empty() || fleet_datetime().is_empty() || fleet_commander_id().is_none(),
                            onclick: move |_| {
                                // TODO: Implement fleet creation
                                tracing::info!("Create fleet: {} at {} UTC", fleet_name(), fleet_datetime());
                            },
                            "Create Fleet"
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
