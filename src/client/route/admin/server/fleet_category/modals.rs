use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{client::component::FullScreenModal, model::ping_format::PingFormatDto};

use super::{
    duration::{format_duration, parse_duration, validate_duration_input},
    form_fields::{FleetCategoryFormFields, FormFieldsData, ValidationErrorsData},
};

#[cfg(feature = "web")]
use crate::client::api::{
    fleet_category::{create_fleet_category, update_fleet_category},
    ping_format::get_ping_formats,
};

/// Duration field values for submission
#[derive(Clone, Default)]
struct DurationFields {
    ping_format_id: Option<i32>,
    ping_cooldown: Option<Duration>,
    ping_reminder: Option<Duration>,
    max_pre_ping: Option<Duration>,
}

impl ValidationErrorsData {
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
    let mut form_fields = use_signal(FormFieldsData::default);
    let mut submit_data = use_signal(|| (String::new(), DurationFields::default()));
    let mut validation_errors = use_signal(ValidationErrorsData::default);
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut ping_formats = use_signal(|| Vec::<PingFormatDto>::new());

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

    // Reset form when modal opens (clears data from previous use)
    use_effect(move || {
        if show() {
            form_fields.set(FormFieldsData::default());
            submit_data.set((String::new(), DurationFields::default()));
            validation_errors.set(ValidationErrorsData::default());
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
                Some(
                    create_fleet_category(
                        guild_id,
                        ping_format_id,
                        name,
                        durations.ping_cooldown,
                        durations.ping_reminder,
                        durations.max_pre_ping,
                    )
                    .await,
                )
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
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Close modal (data persists for smooth animation)
                    show.set(false);
                    should_submit.set(false);
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
        let errors = ValidationErrorsData {
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
                    form_fields,
                    validation_errors,
                    is_submitting,
                    ping_formats
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
    category_to_edit: Signal<Option<crate::model::fleet::FleetCategoryDto>>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldsData::default);
    let mut submit_data = use_signal(|| (0i32, String::new(), DurationFields::default()));
    let mut validation_errors = use_signal(ValidationErrorsData::default);
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut ping_formats = use_signal(|| Vec::<PingFormatDto>::new());

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

    // Initialize form when modal opens with new data
    use_effect(move || {
        if show() {
            if let Some(category) = category_to_edit() {
                form_fields.set(FormFieldsData {
                    category_name: category.name.clone(),
                    ping_format_id: Some(category.ping_format_id),
                    search_query: String::new(),
                    ping_cooldown_str: category
                        .ping_lead_time
                        .as_ref()
                        .map(|d| format_duration(d))
                        .unwrap_or_default(),
                    ping_reminder_str: category
                        .ping_reminder
                        .as_ref()
                        .map(|d| format_duration(d))
                        .unwrap_or_default(),
                    max_pre_ping_str: category
                        .max_pre_ping
                        .as_ref()
                        .map(|d| format_duration(d))
                        .unwrap_or_default(),
                    active_tab: Default::default(),
                    role_search_query: String::new(),
                    channel_search_query: String::new(),
                    access_roles: Vec::new(),
                    ping_roles: Vec::new(),
                    channels: Vec::new(),
                });
                submit_data.write().0 = category.id;
                validation_errors.set(ValidationErrorsData::default());
                error.set(None);
                should_submit.set(false);
            }
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let (id, name, durations) = submit_data();
            if let Some(ping_format_id) = durations.ping_format_id {
                Some(
                    update_fleet_category(
                        guild_id,
                        id,
                        ping_format_id,
                        name,
                        durations.ping_cooldown,
                        durations.ping_reminder,
                        durations.max_pre_ping,
                    )
                    .await,
                )
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
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Close modal (data persists for smooth animation)
                    show.set(false);
                    should_submit.set(false);
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
        let errors = ValidationErrorsData {
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
                    form_fields,
                    validation_errors,
                    is_submitting,
                    ping_formats
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
