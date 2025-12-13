use dioxus::prelude::*;

use super::duration::validate_duration_input;

/// Form field values
#[derive(Clone, Default, PartialEq)]
pub struct FormFieldsData {
    pub category_name: String,
    pub ping_cooldown_str: String,
    pub ping_reminder_str: String,
    pub max_pre_ping_str: String,
}

/// Validation errors for duration fields
#[derive(Clone, Default, PartialEq)]
pub struct ValidationErrorsData {
    pub ping_cooldown: Option<String>,
    pub ping_reminder: Option<String>,
    pub max_pre_ping: Option<String>,
}

/// Reusable form fields component for fleet category forms
#[component]
pub fn FleetCategoryFormFields(
    form_fields: Signal<FormFieldsData>,
    validation_errors: Signal<ValidationErrorsData>,
    is_submitting: bool,
) -> Element {
    rsx! {
        // Category Name Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Category Name"
                }
            }
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "e.g., Structure Timers",
                value: "{form_fields().category_name}",
                oninput: move |evt| {
                    form_fields.write().category_name = evt.value();
                },
                disabled: is_submitting,
                required: true,
            }
        }

        // Ping Cooldown Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Ping Cooldown (optional)"
                }
            }
            input {
                r#type: "text",
                class: if validation_errors().ping_cooldown.is_some() { "input input-bordered input-error w-full" } else { "input input-bordered w-full" },
                placeholder: "e.g., 1h, 30m, 1h30m",
                value: "{form_fields().ping_cooldown_str}",
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().ping_cooldown_str = value.clone();
                    validation_errors.write().ping_cooldown = validate_duration_input(&value);
                },
                disabled: is_submitting,
            }
            if let Some(error) = &validation_errors().ping_cooldown {
                div {
                    class: "text-error text-sm mt-1",
                    "{error}"
                }
            }
            label {
                class: "label flex-col items-start gap-1",
                span {
                    class: "label-text-alt",
                    "Minimum amount of time between fleets"
                }
                span {
                    class: "label-text-alt text-xs",
                    "Format: 1h = 1 hour, 30m = 30 minutes, 1h30m = 1.5 hours"
                }
            }
        }

        // Ping Reminder Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Ping Reminder (optional)"
                }
            }
            input {
                r#type: "text",
                class: if validation_errors().ping_reminder.is_some() { "input input-bordered input-error w-full" } else { "input input-bordered w-full" },
                placeholder: "e.g., 15m, 30m",
                value: "{form_fields().ping_reminder_str}",
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().ping_reminder_str = value.clone();
                    validation_errors.write().ping_reminder = validate_duration_input(&value);
                },
                disabled: is_submitting,
            }
            if let Some(error) = &validation_errors().ping_reminder {
                div {
                    class: "text-error text-sm mt-1",
                    "{error}"
                }
            }
            label {
                class: "label flex-col items-start gap-1",
                span {
                    class: "label-text-alt",
                    "Reminder ping before fleet starts"
                }
                span {
                    class: "label-text-alt text-xs",
                    "Format: 1h = 1 hour, 30m = 30 minutes, 1h30m = 1.5 hours"
                }
            }
        }

        // Max Pre-Ping Input
        div {
            class: "form-control w-full gap-3",
            label {
                class: "label",
                span {
                    class: "label-text",
                    "Max Pre-Ping (optional)"
                }
            }
            input {
                r#type: "text",
                class: if validation_errors().max_pre_ping.is_some() { "input input-bordered input-error w-full" } else { "input input-bordered w-full" },
                placeholder: "e.g., 2h, 3h",
                value: "{form_fields().max_pre_ping_str}",
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().max_pre_ping_str = value.clone();
                    validation_errors.write().max_pre_ping = validate_duration_input(&value);
                },
                disabled: is_submitting,
            }
            if let Some(error) = &validation_errors().max_pre_ping {
                div {
                    class: "text-error text-sm mt-1",
                    "{error}"
                }
            }
            label {
                class: "label flex-col items-start gap-1",
                span {
                    class: "label-text-alt",
                    "Maximum advance notice for pings"
                }
                span {
                    class: "label-text-alt text-xs",
                    "Format: 1h = 1 hour, 30m = 30 minutes, 1h30m = 1.5 hours"
                }
            }
        }
    }
}
