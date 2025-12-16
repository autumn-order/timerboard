use chrono::{Datelike, TimeZone, Timelike, Utc};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;

use crate::{
    client::{
        component::{ConfirmationModal, FullScreenModal},
        model::error::ApiError,
        store::user::UserState,
    },
    model::{
        category::{FleetCategoryDetailsDto, FleetCategoryListItemDto},
        discord::DiscordGuildMemberDto,
        fleet::UpdateFleetDto,
    },
};

use super::form_fields::FleetFormFields;

#[cfg(feature = "web")]
use crate::client::api::{
    fleet::{
        delete_fleet, get_category_details, get_fleet, get_fleets, get_guild_members, update_fleet,
    },
    user::get_user_manageable_categories,
};

#[derive(Clone, Copy, PartialEq)]
pub enum ViewEditMode {
    View,
    Edit,
}

/// Combined modal for viewing and editing fleet details
#[component]
pub fn FleetViewEditModal(
    guild_id: u64,
    fleet_id: Signal<Option<i32>>,
    mut show: Signal<bool>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let user_store = use_context::<Store<UserState>>();
    let current_user = user_store.read().user.clone();
    let current_user_id = current_user.as_ref().map(|user| user.discord_id);

    let mut mode = use_signal(|| ViewEditMode::View);
    let mut fleet_data = use_signal(|| None::<Result<crate::model::fleet::FleetDto, ApiError>>);
    let mut category_details = use_signal(|| None::<Result<FleetCategoryDetailsDto, ApiError>>);
    let mut guild_members = use_signal(|| None::<Result<Vec<DiscordGuildMemberDto>, ApiError>>);
    let mut manageable_categories =
        use_signal(|| None::<Result<Vec<FleetCategoryListItemDto>, ApiError>>);
    let mut existing_fleets =
        use_signal(|| None::<Result<Vec<crate::model::fleet::FleetListItemDto>, ApiError>>);

    // Track selected category (can be changed via dropdown in edit mode)
    let mut selected_category_id = use_signal(|| 0);

    // Refetch trigger for fleet data
    let mut fleet_refetch_trigger = use_signal(|| 0u32);

    // Form state for editing
    let mut fleet_name = use_signal(|| String::new());
    let mut fleet_datetime = use_signal(|| String::new());
    let mut original_fleet_time = use_signal(|| None::<chrono::DateTime<Utc>>);
    let mut fleet_commander_id = use_signal(|| None::<u64>);
    let mut fleet_description = use_signal(|| String::new());
    let mut field_values = use_signal(|| HashMap::<i32, String>::new());

    // Submission state
    let mut is_submitting = use_signal(|| false);
    let mut submission_error = use_signal(|| None::<String>);

    // Delete modal state
    let mut show_delete_modal = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Validation warnings
    let mut validation_warnings = use_signal(|| Vec::<String>::new());

    // Permission check: user can manage if they are admin, have manage permission, or are the fleet commander
    let can_manage = use_memo(move || {
        if let Some(user) = &current_user {
            if user.admin {
                return true;
            }
            if let Some(Ok(fleet)) = fleet_data() {
                // Check if user is the fleet commander
                if Some(fleet.commander_id) == current_user_id {
                    return true;
                }
                // Check if user has manage permission for this category
                if let Some(Ok(categories)) = manageable_categories() {
                    return categories.iter().any(|cat| cat.id == fleet.category_id);
                }
            }
        }
        false
    });

    // Check if more than 1 hour has elapsed since fleet start time
    let is_fleet_edit_locked = use_memo(move || {
        if let Some(Ok(fleet)) = fleet_data() {
            let now = Utc::now();
            let one_hour_after_start = fleet.fleet_time + chrono::Duration::hours(1);
            now > one_hour_after_start
        } else {
            false
        }
    });

    // Reset form when modal opens with a new fleet
    use_effect(move || {
        if show() && fleet_id().is_some() {
            mode.set(ViewEditMode::View);
            submission_error.set(None);
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

    // Fetch fleet data
    #[cfg(feature = "web")]
    {
        let fetch_fleet_future = use_resource(move || async move {
            let _ = fleet_refetch_trigger(); // Read trigger to re-run when it changes
            if show() {
                if let Some(id) = fleet_id() {
                    Some(get_fleet(guild_id, id).await)
                } else {
                    None
                }
            } else {
                None
            }
        });

        use_effect(move || {
            if let Some(Some(result)) = fetch_fleet_future.read_unchecked().as_ref() {
                match result {
                    Ok(fleet) => {
                        // Populate form fields from fleet data
                        fleet_name.set(fleet.name.clone());
                        let dt = fleet.fleet_time;
                        fleet_datetime.set(format!(
                            "{:04}-{:02}-{:02} {:02}:{:02}",
                            dt.year(),
                            dt.month(),
                            dt.day(),
                            dt.hour(),
                            dt.minute()
                        ));
                        original_fleet_time.set(Some(fleet.fleet_time));
                        fleet_commander_id.set(Some(fleet.commander_id));
                        fleet_description.set(fleet.description.clone().unwrap_or_default());
                        selected_category_id.set(fleet.category_id);

                        // Convert field_values from field_name->value to field_id->value
                        // We'll need to get the field IDs from category details
                        fleet_data.set(Some(Ok(fleet.clone())));
                    }
                    Err(err) => {
                        tracing::error!("Failed to fetch fleet: {}", err);
                        fleet_data.set(Some(Err(err.clone())));
                    }
                }
            }
        });
    }

    // Fetch category details (re-fetches when selected_category_id changes in edit mode)
    #[cfg(feature = "web")]
    {
        let fetch_category_future =
            use_resource(use_reactive!(|selected_category_id| async move {
                if show() && selected_category_id() != 0 {
                    Some(get_category_details(guild_id, selected_category_id()).await)
                } else {
                    None
                }
            }));

        use_effect(move || {
            if let Some(Some(result)) = fetch_category_future.read_unchecked().as_ref() {
                match result {
                    Ok(details) => {
                        category_details.set(Some(Ok(details.clone())));

                        // Only map field values if we're viewing the original fleet category
                        // If category has been changed, leave field_values empty (user must fill new fields)
                        if let Some(Ok(fleet)) = fleet_data() {
                            if selected_category_id() == fleet.category_id {
                                // Convert field_values from name->value to id->value for the original category
                                let mut id_to_value_map = HashMap::new();
                                for field in &details.fields {
                                    if let Some(value) = fleet.field_values.get(&field.name) {
                                        id_to_value_map.insert(field.id, value.clone());
                                    }
                                }
                                field_values.set(id_to_value_map);
                            }
                            // If category changed, field_values are already cleared by the form dropdown onchange
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to fetch category details: {}", err);
                        category_details.set(Some(Err(err.clone())));
                    }
                }
            }
        });
    }

    // Fetch guild members
    #[cfg(feature = "web")]
    {
        let fetch_members_future =
            use_resource(move || async move { get_guild_members(guild_id).await });

        use_effect(move || match &*fetch_members_future.read_unchecked() {
            Some(Ok(members)) => {
                guild_members.set(Some(Ok(members.clone())));
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch guild members: {}", err);
                guild_members.set(Some(Err(err.clone())));
            }
            None => (),
        });
    }

    // Fetch existing fleets for validation
    #[cfg(feature = "web")]
    {
        let future = use_resource(use_reactive!(|selected_category_id| async move {
            // Fetch a large number of fleets to check for conflicts
            get_fleets(guild_id, 0, 1000).await
        }));

        use_effect(move || match &*future.read_unchecked() {
            Some(Ok(paginated)) => {
                let fleets_in_category: Vec<_> = paginated
                    .fleets
                    .iter()
                    .filter(|f| f.category_id == selected_category_id())
                    .cloned()
                    .collect();
                existing_fleets.set(Some(Ok(fleets_in_category)));
            }
            Some(Err(err)) => {
                tracing::error!("Failed to fetch fleets: {}", err);
                existing_fleets.set(Some(Err(err.clone())));
            }
            None => (),
        });
    }

    // Validate fleet datetime against category rules (only in edit mode)
    use_effect(use_reactive!(|(
        mode,
        fleet_datetime,
        original_fleet_time,
        category_details,
        existing_fleets,
        fleet_id,
    )| {
        let mut warnings = Vec::new();

        if mode() == ViewEditMode::Edit {
            if let Some(Ok(details)) = category_details() {
                // Parse the fleet datetime
                if let Ok(parsed_datetime) =
                    chrono::NaiveDateTime::parse_from_str(&fleet_datetime(), "%Y-%m-%d %H:%M")
                {
                    let fleet_time = Utc.from_utc_datetime(&parsed_datetime);
                    let now = Utc::now();

                    // Check max_pre_ping (maximum advance scheduling)
                    // Only validate if the new time is in the future
                    if let Some(max_pre_ping) = details.max_pre_ping {
                        if fleet_time > now {
                            let max_schedule_time = now + max_pre_ping;
                            if fleet_time > max_schedule_time {
                                let hours = max_pre_ping.num_hours();
                                warnings.push(format!(
                                    "Fleet is scheduled more than {} hour{} in advance",
                                    hours,
                                    if hours == 1 { "" } else { "s" }
                                ));
                            }
                        }
                    }

                    // Check ping_lead_time (minimum gap between fleets)
                    if let (Some(ping_lead_time), Some(Ok(fleets)), Some(current_fleet_id)) =
                        (details.ping_lead_time, existing_fleets(), fleet_id())
                    {
                        for existing_fleet in fleets {
                            // Skip the current fleet being edited
                            if existing_fleet.id == current_fleet_id {
                                continue;
                            }

                            let time_diff = if fleet_time > existing_fleet.fleet_time {
                                fleet_time - existing_fleet.fleet_time
                            } else {
                                existing_fleet.fleet_time - fleet_time
                            };

                            if time_diff < ping_lead_time {
                                let hours = ping_lead_time.num_hours();
                                let minutes = (ping_lead_time.num_minutes() % 60) as i64;
                                let time_str = if hours > 0 {
                                    if minutes > 0 {
                                        format!(
                                            "{} hour{} {} minute{}",
                                            hours,
                                            if hours == 1 { "" } else { "s" },
                                            minutes,
                                            if minutes == 1 { "" } else { "s" }
                                        )
                                    } else {
                                        format!(
                                            "{} hour{}",
                                            hours,
                                            if hours == 1 { "" } else { "s" }
                                        )
                                    }
                                } else {
                                    format!(
                                        "{} minute{}",
                                        minutes,
                                        if minutes == 1 { "" } else { "s" }
                                    )
                                };
                                warnings.push(format!(
                                    "Fleet \"{}\" at {} is within {} of this fleet",
                                    existing_fleet.name,
                                    existing_fleet.fleet_time.format("%Y-%m-%d %H:%M UTC"),
                                    time_str
                                ));
                                break; // Only show one warning to avoid clutter
                            }
                        }
                    }
                }
            }
        }

        validation_warnings.set(warnings);
    }));

    // Handle fleet update submission
    #[cfg(feature = "web")]
    let update_future = use_resource(move || async move {
        if is_submitting() && mode() == ViewEditMode::Edit {
            if let Some(id) = fleet_id() {
                let dto = UpdateFleetDto {
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
                Some(update_fleet(guild_id, id, dto).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = update_future.read_unchecked().as_ref() {
            match result {
                Ok(_fleet) => {
                    tracing::info!("Fleet updated successfully");
                    is_submitting.set(false);
                    mode.set(ViewEditMode::View);
                    submission_error.set(None);
                    // Trigger refetch in parent
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Refetch fleet data to show updated values
                    fleet_refetch_trigger.set(fleet_refetch_trigger() + 1);
                }
                Err(err) => {
                    tracing::error!("Failed to update fleet: {}", err);
                    submission_error.set(Some(format!("Failed to update fleet: {}", err)));
                    is_submitting.set(false);
                }
            }
        }
    });

    // Handle fleet deletion
    #[cfg(feature = "web")]
    let delete_future = use_resource(move || async move {
        if is_deleting() {
            if let Some(id) = fleet_id() {
                Some(delete_fleet(guild_id, id).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = delete_future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    tracing::info!("Fleet deleted successfully");
                    is_deleting.set(false);
                    show_delete_modal.set(false);
                    show.set(false);
                    // Trigger refetch in parent
                    refetch_trigger.set(refetch_trigger() + 1);
                }
                Err(err) => {
                    tracing::error!("Failed to delete fleet: {}", err);
                    is_deleting.set(false);
                }
            }
        }
    });

    let modal_title = use_memo(move || {
        if let Some(Ok(fleet)) = fleet_data() {
            match mode() {
                ViewEditMode::View => format!("Fleet: {}", fleet.name),
                ViewEditMode::Edit => format!("Edit Fleet: {}", fleet.name),
            }
        } else {
            "Fleet Details".to_string()
        }
    });

    rsx! {
        FullScreenModal {
            show,
            title: "{modal_title}",
            prevent_close: is_submitting() || is_deleting(),
            div {
                class: "space-y-4 overflow-y-auto max-h-[calc(100vh-200px)] sm:max-h-[calc(90vh-200px)]",

                match mode() {
                    ViewEditMode::View => rsx! {
                        if let Some(Ok(fleet)) = fleet_data() {
                            // View Mode - Display fleet information in same format as form
                            div {
                                class: "space-y-4",

                                // Category and Fleet Info
                                div {
                                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                                    div {
                                        class: "flex flex-col gap-2",
                                        label {
                                            class: "label",
                                            span { class: "label-text", "Category" }
                                        }
                                        div {
                                            class: "input input-bordered w-full flex items-center bg-base-200",
                                            "{fleet.category_name}"
                                        }
                                    }

                                    div {
                                        class: "flex flex-col gap-2",
                                        label {
                                            class: "label",
                                            span { class: "label-text", "Fleet Name" }
                                        }
                                        div {
                                            class: "input input-bordered w-full flex items-center bg-base-200",
                                            "{fleet.name}"
                                        }
                                    }

                                    div {
                                        class: "flex flex-col gap-2",
                                        label {
                                            class: "label",
                                            span { class: "label-text", "Fleet Date & Time" }
                                        }
                                        div {
                                            class: "input input-bordered w-full flex items-center bg-base-200 font-mono",
                                            {fleet.fleet_time.format("%Y-%m-%d %H:%M").to_string()}
                                            span { class: "ml-2 text-sm opacity-60", "(UTC)" }
                                        }
                                    }

                                    div {
                                        class: "flex flex-col gap-2",
                                        label {
                                            class: "label",
                                            span { class: "label-text", "Fleet Commander" }
                                        }
                                        div {
                                            class: "input input-bordered w-full flex items-center bg-base-200",
                                            "{fleet.commander_name}"
                                        }
                                    }
                                }

                                // Ping Format Fields
                                if !fleet.field_values.is_empty() {
                                    div {
                                        class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                        for (field_name, field_value) in &fleet.field_values {
                                            div {
                                                key: "{field_name}",
                                                class: "flex flex-col gap-2",
                                                label {
                                                    class: "label",
                                                    span { class: "label-text", "{field_name}" }
                                                }
                                                div {
                                                    class: "input input-bordered w-full flex items-center bg-base-200",
                                                    "{field_value}"
                                                }
                                            }
                                        }
                                    }
                                }

                                // Description
                                div {
                                    class: "space-y-2",
                                    h3 {
                                        class: "text-lg font-bold",
                                        "Description"
                                    }
                                    div {
                                        class: "textarea textarea-bordered min-h-48 max-h-96 w-full bg-base-200 overflow-y-auto prose prose-sm max-w-none",
                                        if let Some(desc) = &fleet.description {
                                            if !desc.is_empty() {
                                                // Parse markdown to HTML
                                                {
                                                    let options = Options::all();
                                                    let parser = Parser::new_ext(desc, options);
                                                    let mut html_output = String::new();
                                                    html::push_html(&mut html_output, parser);
                                                    rsx! {
                                                        div {
                                                            dangerous_inner_html: "{html_output}"
                                                        }
                                                    }
                                                }
                                            } else {
                                                span { class: "opacity-50 italic", "No description provided" }
                                            }
                                        } else {
                                            span { class: "opacity-50 italic", "No description provided" }
                                        }
                                    }
                                }

                                // Action Buttons
                                div {
                                    class: "flex gap-2 justify-end pt-4",
                                    button {
                                        class: "btn",
                                        onclick: move |_| show.set(false),
                                        "Close"
                                    }
                                    if can_manage() {
                                        button {
                                            class: "btn btn-error",
                                            onclick: move |_| show_delete_modal.set(true),
                                            "Delete"
                                        }
                                        button {
                                            class: "btn btn-primary",
                                            disabled: is_fleet_edit_locked(),
                                            onclick: move |_| {
                                                mode.set(ViewEditMode::Edit);
                                                submission_error.set(None);
                                            },
                                            "Edit"
                                        }
                                    }
                                }
                            }
                        } else if let Some(Err(_)) = fleet_data() {
                            div {
                                class: "text-center py-8",
                                p {
                                    class: "text-error",
                                    "Failed to load fleet details"
                                }
                                button {
                                    class: "btn btn-ghost mt-4",
                                    onclick: move |_| show.set(false),
                                    "Close"
                                }
                            }
                        } else {
                            div {
                                class: "flex items-center justify-center py-8",
                                span { class: "loading loading-spinner loading-lg" }
                            }
                        }
                    },
                    ViewEditMode::Edit => rsx! {
                        // Edit Mode - Use shared form fields with category selection
                        {
                            // Determine if we should allow past times and set minimum datetime
                            let (allow_past, min_dt) = if let Some(original_time) = original_fleet_time() {
                                // Only enforce minimum if original time is in the past
                                if original_time < Utc::now() {
                                    (true, Some(original_time))
                                } else {
                                    // Original time is in the future, so normal validation applies
                                    (false, None)
                                }
                            } else {
                                (false, None)
                            };

                            rsx! {
                                FleetFormFields {
                                    guild_id,
                                    fleet_name,
                                    fleet_datetime,
                                    fleet_commander_id,
                                    fleet_description,
                                    field_values,
                                    category_details,
                                    guild_members,
                                    is_submitting: is_submitting(),
                                    current_user_id,
                                    selected_category_id: Some(selected_category_id),
                                    manageable_categories: Some(manageable_categories),
                                    allow_past_time: allow_past,
                                    min_datetime: min_dt,
                                }
                            }
                        }

                        // Validation warnings
                        if !validation_warnings().is_empty() {
                            for warning in validation_warnings() {
                                div {
                                    key: "{warning}",
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
                                            d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                                        }
                                    }
                                    span { "{warning}" }
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

                        // Action Buttons
                        div {
                            class: "flex gap-2 justify-end pt-4",
                            button {
                                class: "btn",
                                disabled: is_submitting(),
                                onclick: move |_| {
                                    mode.set(ViewEditMode::View);
                                    submission_error.set(None);
                                    // Reset form to original values
                                    if let Some(Ok(fleet)) = fleet_data() {
                                        fleet_name.set(fleet.name.clone());
                                        let dt = fleet.fleet_time;
                                        fleet_datetime.set(format!(
                                            "{:04}-{:02}-{:02} {:02}:{:02}",
                                            dt.year(),
                                            dt.month(),
                                            dt.day(),
                                            dt.hour(),
                                            dt.minute()
                                        ));
                                        fleet_commander_id.set(Some(fleet.commander_id));
                                        fleet_description.set(fleet.description.clone().unwrap_or_default());
                                        selected_category_id.set(fleet.category_id);
                                        original_fleet_time.set(Some(fleet.fleet_time));
                                        // Reset field values (will be repopulated by category details effect)
                                        field_values.set(HashMap::new());
                                    }
                                },
                                "Cancel"
                            }
                            button {
                                class: "btn btn-primary",
                                disabled: fleet_name().is_empty() || fleet_datetime().is_empty() || fleet_commander_id().is_none() || is_submitting() || !validation_warnings().is_empty(),
                                onclick: move |_| {
                                    is_submitting.set(true);
                                    submission_error.set(None);
                                },
                                if is_submitting() {
                                    span { class: "loading loading-spinner loading-sm" }
                                    " Saving..."
                                } else {
                                    "Save Changes"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Delete Confirmation Modal
        ConfirmationModal {
            show: show_delete_modal,
            title: "Delete Fleet".to_string(),
            message: rsx!(
                if let Some(Ok(fleet)) = fleet_data() {
                    p {
                        class: "py-4",
                        "Are you sure you want to delete the fleet "
                        span { class: "font-bold", "\"{fleet.name}\"" }
                        "? This action cannot be undone."
                    }
                }
            ),
            confirm_text: "Delete".to_string(),
            confirm_class: "btn-error".to_string(),
            is_processing: is_deleting(),
            processing_text: "Deleting...".to_string(),
            on_confirm: move |_| {
                is_deleting.set(true);
            },
        }
    }
}
