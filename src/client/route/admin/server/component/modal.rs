use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::component::modal::FullScreenModal,
    model::{ping_format::PingFormatDto, ping_group::PingGroupDto},
};

use super::{
    super::ValidationErrorData,
    duration::{format_duration, parse_duration, validate_duration_input},
    form_field::{AccessRoleData, ChannelData, FleetCategoryFormFields, FormFieldData, RoleData},
};

#[cfg(feature = "web")]
use crate::{
    client::api::{
        category::{create_fleet_category, get_fleet_category_by_id, update_fleet_category},
        ping_format::get_ping_formats,
        ping_group::get_paginated_ping_groups,
    },
    model::category::{
        CreateFleetCategoryDto, FleetCategoryAccessRoleDto, FleetCategoryChannelDto,
        FleetCategoryPingRoleDto, UpdateFleetCategoryDto,
    },
};

/// Duration field values for submission
#[derive(Clone, Default)]
struct DurationFields {
    ping_format_id: Option<i32>,
    ping_group_id: Option<i32>,
    ping_cooldown: Option<Duration>,
    ping_reminder: Option<Duration>,
    max_pre_ping: Option<Duration>,
}

impl ValidationErrorData {
    fn has_errors(&self) -> bool {
        self.ping_cooldown.is_some() || self.ping_reminder.is_some() || self.max_pre_ping.is_some()
    }
}

#[component]
pub fn CreateCategoryModal(
    guild_id: u64,
    mut show: Signal<bool>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldData::default);
    let mut submit_data = use_signal(|| (String::new(), DurationFields::default()));
    let mut validation_errors = use_signal(ValidationErrorData::default);
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut ping_formats = use_signal(Vec::<PingFormatDto>::new);
    let mut ping_groups = use_signal(Vec::<PingGroupDto>::new);

    // Fetch ping formats when modal opens
    #[cfg(feature = "web")]
    let ping_formats_future = use_resource(move || async move {
        if show() {
            get_ping_formats(guild_id, 0, 100).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = ping_formats_future.read_unchecked().as_ref() {
            ping_formats.set(result.ping_formats.clone());
        }
    });

    // Fetch ping groups when modal opens
    #[cfg(feature = "web")]
    let ping_groups_future = use_resource(move || async move {
        if show() {
            get_paginated_ping_groups(guild_id, 0, 100).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = ping_groups_future.read_unchecked().as_ref() {
            ping_groups.set(result.items.clone());
        }
    });

    // Reset form when modal opens (clears data from previous use)
    use_effect(move || {
        if show() {
            form_fields.set(FormFieldData::default());
            submit_data.set((String::new(), DurationFields::default()));
            validation_errors.set(ValidationErrorData::default());
            should_submit.set(false);
            error.set(None);
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let (name, durations) = submit_data();
            if let Some(ping_format_id) = durations.ping_format_id {
                // Convert form fields to DTOs (server will enrich with names/colors)
                let access_roles: Vec<FleetCategoryAccessRoleDto> = form_fields()
                    .access_roles
                    .iter()
                    .map(|r| FleetCategoryAccessRoleDto {
                        role_id: r.role.id,
                        role_name: String::new(),  // Server will populate
                        role_color: String::new(), // Server will populate
                        position: 0,               // Server will populate
                        can_view: r.can_view,
                        can_create: r.can_create,
                        can_manage: r.can_manage,
                    })
                    .collect();

                let ping_roles: Vec<FleetCategoryPingRoleDto> = form_fields()
                    .ping_roles
                    .iter()
                    .map(|r| FleetCategoryPingRoleDto {
                        role_id: r.id,
                        role_name: String::new(),  // Server will populate
                        role_color: String::new(), // Server will populate
                        position: 0,               // Server will populate
                    })
                    .collect();

                let channels: Vec<FleetCategoryChannelDto> = form_fields()
                    .channels
                    .iter()
                    .map(|c| FleetCategoryChannelDto {
                        channel_id: c.id,
                        channel_name: String::new(), // Server will populate
                        position: 0,                 // Server will populate
                    })
                    .collect();

                let dto = CreateFleetCategoryDto {
                    ping_format_id,
                    name,
                    ping_group_id: durations.ping_group_id,
                    ping_lead_time: durations.ping_cooldown,
                    ping_reminder: durations.ping_reminder,
                    max_pre_ping: durations.max_pre_ping,
                    access_roles,
                    ping_roles,
                    channels,
                };

                Some(create_fleet_category(guild_id, dto).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    // Reset form data
                    form_fields.set(FormFieldData::default());
                    submit_data.set((String::new(), DurationFields::default()));
                    validation_errors.set(ValidationErrorData::default());
                    error.set(None);
                    should_submit.set(false);
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);

                    show.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to create category: {}", err);
                    error.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let fields = form_fields();
        if fields.category_name.trim().is_empty() {
            error.set(Some("Category name is required".to_string()));
            return;
        }

        if fields.ping_format_id.is_none() {
            error.set(Some("Ping format is required".to_string()));
            return;
        }

        // Validate all duration fields before submitting
        let errors = ValidationErrorData {
            ping_cooldown: validate_duration_input(&fields.ping_cooldown_str),
            ping_reminder: validate_duration_input(&fields.ping_reminder_str),
            max_pre_ping: validate_duration_input(&fields.max_pre_ping_str),
        };

        validation_errors.set(errors.clone());

        // Only submit if all validations pass
        if !errors.has_errors() {
            error.set(None);
            let durations = DurationFields {
                ping_format_id: fields.ping_format_id,
                ping_group_id: fields.ping_group_id,
                ping_cooldown: parse_duration(&fields.ping_cooldown_str),
                ping_reminder: parse_duration(&fields.ping_reminder_str),
                max_pre_ping: parse_duration(&fields.max_pre_ping_str),
            };
            submit_data.set((fields.category_name, durations));
            should_submit.set(true);
        } else {
            error.set(Some("Please fix the validation errors above".to_string()));
        }
    };

    let is_submitting = should_submit();
    let has_validation_errors = validation_errors().has_errors();

    rsx!(
        FullScreenModal {
            show,
            title: "Create Fleet Category".to_string(),
            prevent_close: is_submitting,
            class: None,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                FleetCategoryFormFields {
                    guild_id,
                    form_fields,
                    validation_errors,
                    is_submitting,
                    ping_formats,
                    ping_groups
                }

                // Error Message
                if let Some(err) = error() {
                    div {
                        class: "alert alert-error mt-4",
                        span { "{err}" }
                    }
                }

                // Modal Actions
                div {
                    class: "modal-action",
                    button {
                        r#type: "button",
                        class: "btn",
                        onclick: move |_| show.set(false),
                        disabled: is_submitting,
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: is_submitting || has_validation_errors,
                        if is_submitting {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Creating..."
                        } else {
                            "Create"
                        }
                    }
                }
            }
        }
    )
}

#[component]
pub fn EditCategoryModal(
    guild_id: u64,
    mut show: Signal<bool>,
    category_id: Signal<Option<i32>>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldData::default);
    let mut submit_data = use_signal(|| (0i32, String::new(), DurationFields::default()));
    let mut validation_errors = use_signal(ValidationErrorData::default);
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut ping_formats = use_signal(Vec::<PingFormatDto>::new);
    let mut ping_groups = use_signal(Vec::<PingGroupDto>::new);

    // Fetch ping formats when modal opens
    #[cfg(feature = "web")]
    let ping_formats_future = use_resource(move || async move {
        if show() {
            get_ping_formats(guild_id, 0, 100).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = ping_formats_future.read_unchecked().as_ref() {
            ping_formats.set(result.ping_formats.clone());
        }
    });

    // Fetch ping groups when modal opens
    #[cfg(feature = "web")]
    let ping_groups_future = use_resource(move || async move {
        if show() {
            get_paginated_ping_groups(guild_id, 0, 100).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = ping_groups_future.read_unchecked().as_ref() {
            ping_groups.set(result.items.clone());
        }
    });

    // Fetch full category details - refreshes when refetch is true
    #[cfg(feature = "web")]
    let category_future = use_resource(move || async move {
        if let Some(cat_id) = category_id() {
            get_fleet_category_by_id(guild_id, cat_id).await.ok()
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(category)) = category_future.read_unchecked().as_ref() {
            form_fields.set(FormFieldData {
                category_name: category.name.clone(),
                ping_format_id: Some(category.ping_format_id),
                ping_group_id: category.ping_group_id,
                search_query: String::new(),
                ping_cooldown_str: category
                    .ping_lead_time
                    .as_ref()
                    .map(format_duration)
                    .unwrap_or_default(),
                ping_reminder_str: category
                    .ping_reminder
                    .as_ref()
                    .map(format_duration)
                    .unwrap_or_default(),
                max_pre_ping_str: category
                    .max_pre_ping
                    .as_ref()
                    .map(format_duration)
                    .unwrap_or_default(),
                active_tab: Default::default(),
                role_search_query: String::new(),
                channel_search_query: String::new(),
                access_roles: category
                    .access_roles
                    .iter()
                    .map(|ar| AccessRoleData {
                        role: RoleData {
                            id: ar.role_id,
                            name: ar.role_name.clone(),
                            color: ar.role_color.clone(),
                            position: ar.position,
                        },
                        can_view: ar.can_view,
                        can_create: ar.can_create,
                        can_manage: ar.can_manage,
                    })
                    .collect(),
                ping_roles: category
                    .ping_roles
                    .iter()
                    .map(|pr| RoleData {
                        id: pr.role_id,
                        name: pr.role_name.clone(),
                        color: pr.role_color.clone(),
                        position: pr.position,
                    })
                    .collect(),
                channels: category
                    .channels
                    .iter()
                    .map(|c| ChannelData {
                        id: c.channel_id,
                        name: c.channel_name.clone(),
                        position: c.position,
                    })
                    .collect(),
            });
            submit_data.write().0 = category.id;
            validation_errors.set(ValidationErrorData::default());
            error.set(None);
        }
    });

    // Reset form when modal opens and trigger fresh fetch
    use_effect(move || {
        if show() {
            // Reset submission state and errors
            should_submit.set(false);
            error.set(None);
            validation_errors.set(ValidationErrorData::default());

            // If there's no category_id, reset form fields (shouldn't happen in edit modal)
            if category_id().is_none() {
                form_fields.set(FormFieldData::default());
                submit_data.set((0i32, String::new(), DurationFields::default()));
            }
            // The category_future resource will populate the form fields
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let (id, name, durations) = submit_data();
            if let Some(ping_format_id) = durations.ping_format_id {
                // Convert form fields to DTOs (server will enrich with names/colors)
                let access_roles: Vec<FleetCategoryAccessRoleDto> = form_fields()
                    .access_roles
                    .iter()
                    .map(|r| FleetCategoryAccessRoleDto {
                        role_id: r.role.id,
                        role_name: String::new(),  // Server will populate
                        role_color: String::new(), // Server will populate
                        position: 0,               // Server will populate
                        can_view: r.can_view,
                        can_create: r.can_create,
                        can_manage: r.can_manage,
                    })
                    .collect();

                let ping_roles: Vec<FleetCategoryPingRoleDto> = form_fields()
                    .ping_roles
                    .iter()
                    .map(|r| FleetCategoryPingRoleDto {
                        role_id: r.id,
                        role_name: String::new(),  // Server will populate
                        role_color: String::new(), // Server will populate
                        position: 0,               // Server will populate
                    })
                    .collect();

                let channels: Vec<FleetCategoryChannelDto> = form_fields()
                    .channels
                    .iter()
                    .map(|c| FleetCategoryChannelDto {
                        channel_id: c.id,
                        channel_name: String::new(), // Server will populate
                        position: 0,                 // Server will populate
                    })
                    .collect();

                let dto = UpdateFleetCategoryDto {
                    ping_format_id,
                    name,
                    ping_group_id: durations.ping_group_id,
                    ping_lead_time: durations.ping_cooldown,
                    ping_reminder: durations.ping_reminder,
                    max_pre_ping: durations.max_pre_ping,
                    access_roles,
                    ping_roles,
                    channels,
                };

                Some(update_fleet_category(guild_id, id, dto).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    // Reset form data
                    form_fields.set(FormFieldData::default());
                    submit_data.set((0i32, String::new(), DurationFields::default()));
                    validation_errors.set(ValidationErrorData::default());
                    error.set(None);
                    should_submit.set(false);
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);

                    show.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to update category: {}", err);
                    error.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let fields = form_fields();
        if fields.category_name.trim().is_empty() {
            error.set(Some("Category name is required".to_string()));
            return;
        }

        if fields.ping_format_id.is_none() {
            error.set(Some("Ping format is required".to_string()));
            return;
        }

        // Validate all duration fields before submitting
        let errors = ValidationErrorData {
            ping_cooldown: validate_duration_input(&fields.ping_cooldown_str),
            ping_reminder: validate_duration_input(&fields.ping_reminder_str),
            max_pre_ping: validate_duration_input(&fields.max_pre_ping_str),
        };

        validation_errors.set(errors.clone());

        // Only submit if all validations pass
        if !errors.has_errors() {
            error.set(None);
            let durations = DurationFields {
                ping_format_id: fields.ping_format_id,
                ping_group_id: fields.ping_group_id,
                ping_cooldown: parse_duration(&fields.ping_cooldown_str),
                ping_reminder: parse_duration(&fields.ping_reminder_str),
                max_pre_ping: parse_duration(&fields.max_pre_ping_str),
            };
            let id = submit_data().0;
            submit_data.set((id, fields.category_name, durations));
            should_submit.set(true);
        } else {
            error.set(Some("Please fix the validation errors above".to_string()));
        }
    };

    let is_submitting = should_submit();
    let has_validation_errors = validation_errors().has_errors();

    rsx!(
        FullScreenModal {
            show,
            title: "Edit Fleet Category".to_string(),
            prevent_close: is_submitting,
            class: None,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                FleetCategoryFormFields {
                    guild_id,
                    form_fields,
                    validation_errors,
                    is_submitting,
                    ping_formats,
                    ping_groups
                }

                // Error Message
                if let Some(err) = error() {
                    div {
                        class: "alert alert-error mt-4",
                        span { "{err}" }
                    }
                }

                // Modal Actions
                div {
                    class: "modal-action",
                    button {
                        r#type: "button",
                        class: "btn",
                        onclick: move |_| show.set(false),
                        disabled: is_submitting,
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: is_submitting || has_validation_errors,
                        if is_submitting {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Updating..."
                        } else {
                            "Update"
                        }
                    }
                }
            }
        }
    )
}
