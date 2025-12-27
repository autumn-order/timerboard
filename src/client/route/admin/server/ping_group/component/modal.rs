use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::component::Modal,
    model::ping_group::{CreatePingGroupDto, UpdatePingGroupDto},
};

#[cfg(feature = "web")]
use crate::client::api::ping_group::{create_ping_group, update_ping_group};

use crate::client::route::admin::server::component::duration::{
    format_duration, parse_duration, validate_duration_input,
};

/// Form field values
#[derive(Clone, Default)]
struct FormFieldsData {
    name: String,
    cooldown_str: String,
}

#[component]
pub fn CreatePingGroupModal(
    guild_id: u64,
    mut show: Signal<bool>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldsData::default);
    let mut submit_data = use_signal(|| CreatePingGroupDto {
        name: String::new(),
        cooldown: None,
    });
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut cooldown_error = use_signal(|| None::<String>);

    // Reset form when modal opens
    use_effect(move || {
        if show() {
            form_fields.set(FormFieldsData::default());
            submit_data.set(CreatePingGroupDto {
                name: String::new(),
                cooldown: None,
            });
            should_submit.set(false);
            error.set(None);
            cooldown_error.set(None);
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let payload = submit_data();
            Some(create_ping_group(guild_id, payload).await)
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
                    form_fields.set(FormFieldsData::default());
                    submit_data.set(CreatePingGroupDto {
                        name: String::new(),
                        cooldown: None,
                    });
                    error.set(None);
                    cooldown_error.set(None);
                    should_submit.set(false);
                    refetch_trigger.set(refetch_trigger() + 1);
                    show.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to create ping group: {}", err);
                    error.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let fields = form_fields();
        if fields.name.trim().is_empty() {
            error.set(Some("Ping group name is required".to_string()));
            return;
        }

        // Validate cooldown
        let cooldown_validation = validate_duration_input(&fields.cooldown_str);
        if cooldown_validation.is_some() {
            cooldown_error.set(cooldown_validation);
            return;
        }

        // Parse cooldown
        let cooldown = parse_duration(&fields.cooldown_str);

        error.set(None);
        cooldown_error.set(None);
        submit_data.set(CreatePingGroupDto {
            name: fields.name.trim().to_string(),
            cooldown,
        });
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        Modal {
            show,
            title: "Create Ping Group".to_string(),
            prevent_close: is_submitting,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                PingGroupFormFields {
                    form_fields,
                    is_submitting,
                    cooldown_error
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
                        disabled: is_submitting,
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
pub fn EditPingGroupModal(
    guild_id: u64,
    mut show: Signal<bool>,
    group_to_edit: Signal<Option<crate::model::ping_group::PingGroupDto>>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldsData::default);
    let mut submit_data = use_signal(|| {
        (
            0i32,
            UpdatePingGroupDto {
                name: String::new(),
                cooldown: None,
            },
        )
    });
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut cooldown_error = use_signal(|| None::<String>);

    // Initialize form when modal opens with new data
    use_effect(move || {
        if show() {
            if let Some(group) = group_to_edit() {
                let cooldown_str = group
                    .cooldown
                    .as_ref()
                    .map(format_duration)
                    .unwrap_or_default();

                form_fields.set(FormFieldsData {
                    name: group.name.clone(),
                    cooldown_str,
                });
                submit_data.write().0 = group.id;
                error.set(None);
                cooldown_error.set(None);
                should_submit.set(false);
            }
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let (id, payload) = submit_data();
            Some(update_ping_group(guild_id, id, payload).await)
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
                    form_fields.set(FormFieldsData::default());
                    submit_data.set((
                        0i32,
                        UpdatePingGroupDto {
                            name: String::new(),
                            cooldown: None,
                        },
                    ));
                    error.set(None);
                    cooldown_error.set(None);
                    should_submit.set(false);
                    refetch_trigger.set(refetch_trigger() + 1);
                    show.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to update ping group: {}", err);
                    error.set(Some(err.message.clone()));
                    should_submit.set(false);
                }
            }
        }
    });

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let fields = form_fields();
        if fields.name.trim().is_empty() {
            error.set(Some("Ping group name is required".to_string()));
            return;
        }

        // Validate cooldown
        let cooldown_validation = validate_duration_input(&fields.cooldown_str);
        if cooldown_validation.is_some() {
            cooldown_error.set(cooldown_validation);
            return;
        }

        // Parse cooldown
        let cooldown = parse_duration(&fields.cooldown_str);

        error.set(None);
        cooldown_error.set(None);
        let id = submit_data().0;
        submit_data.set((
            id,
            UpdatePingGroupDto {
                name: fields.name.trim().to_string(),
                cooldown,
            },
        ));
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        Modal {
            show,
            title: "Edit Ping Group".to_string(),
            prevent_close: is_submitting,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                PingGroupFormFields {
                    form_fields,
                    is_submitting,
                    cooldown_error
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
                        disabled: is_submitting,
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

#[component]
fn PingGroupFormFields(
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
    mut cooldown_error: Signal<Option<String>>,
) -> Element {
    rsx!(
        // Name field
        div {
            class: "form-control w-full flex flex-col gap-2",
            label {
                class: "label",
                span { class: "label-text", "Ping Group Name" }
            }
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "e.g., Roaming, Strategic",
                value: "{form_fields().name}",
                disabled: is_submitting,
                oninput: move |evt| {
                    form_fields.write().name = evt.value();
                }
            }
        }

        // Cooldown field
        div {
            class: "form-control w-full flex flex-col gap-2",
            label {
                class: "label",
                span { class: "label-text", "Group Cooldown (Optional)" }
            }
            input {
                r#type: "text",
                class: if cooldown_error().is_some() {
                    "input input-bordered input-error w-full"
                } else {
                    "input input-bordered w-full"
                },
                placeholder: "e.g., 1h, 30m, 1h30m (optional)",
                value: "{form_fields().cooldown_str}",
                disabled: is_submitting,
                oninput: move |evt| {
                    let value = evt.value();
                    form_fields.write().cooldown_str = value.clone();
                    cooldown_error.set(validate_duration_input(&value));
                }
            }
            if let Some(err) = cooldown_error() {
                label {
                    class: "label",
                    span { class: "label-text-alt text-error", "{err}" }
                }
            } else {
                label {
                    class: "label",
                    span { class: "label-text-alt opacity-70", "Shared between all fleet categories part of this group" }
                }
            }
        }
    )
}
