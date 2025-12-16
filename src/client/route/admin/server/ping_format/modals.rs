use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::client::component::{Modal, SelectedItemsList};

#[cfg(feature = "web")]
use crate::client::api::ping_format::{create_ping_format, update_ping_format};

/// Form field values
#[derive(Clone, Default)]
struct FormFieldsData {
    name: String,
    fields: Vec<FieldData>,
}

#[derive(Clone, Default)]
struct FieldData {
    id: Option<i32>,
    name: String,
    default_value: String,
}

#[component]
pub fn CreatePingFormatModal(
    guild_id: u64,
    mut show: Signal<bool>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldsData::default);
    let mut submit_data =
        use_signal(|| (String::new(), Vec::<(String, i32, Option<String>)>::new()));
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Reset form when modal opens
    use_effect(move || {
        if show() {
            form_fields.set(FormFieldsData::default());
            submit_data.set((String::new(), Vec::new()));
            should_submit.set(false);
            error.set(None);
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let (name, fields) = submit_data();
            Some(create_ping_format(guild_id, name, fields).await)
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    refetch_trigger.set(refetch_trigger() + 1);
                    show.set(false);
                    should_submit.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to create ping format: {}", err);
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
            error.set(Some("Ping format name is required".to_string()));
            return;
        }

        error.set(None);
        let field_data: Vec<(String, i32, Option<String>)> = fields
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.name.trim().is_empty())
            .map(|(index, f)| {
                let default_val = if f.default_value.trim().is_empty() {
                    None
                } else {
                    Some(f.default_value.clone())
                };
                (f.name.clone(), index as i32, default_val)
            })
            .collect();

        submit_data.set((fields.name, field_data));
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        Modal {
            show,
            title: "Create Ping Format".to_string(),
            prevent_close: is_submitting,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                PingFormatFormFields {
                    form_fields,
                    is_submitting
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
pub fn EditPingFormatModal(
    guild_id: u64,
    mut show: Signal<bool>,
    format_to_edit: Signal<Option<crate::model::ping_format::PingFormatDto>>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut form_fields = use_signal(FormFieldsData::default);
    let mut submit_data = use_signal(|| {
        (
            0i32,
            String::new(),
            Vec::<(Option<i32>, String, i32, Option<String>)>::new(),
        )
    });
    let mut should_submit = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Initialize form when modal opens with new data
    use_effect(move || {
        if show() {
            if let Some(format) = format_to_edit() {
                form_fields.set(FormFieldsData {
                    name: format.name.clone(),
                    fields: format
                        .fields
                        .iter()
                        .map(|f| FieldData {
                            id: Some(f.id),
                            name: f.name.clone(),
                            default_value: f.default_value.clone().unwrap_or_default(),
                        })
                        .collect(),
                });
                submit_data.write().0 = format.id;
                error.set(None);
                should_submit.set(false);
            }
        }
    });

    // Handle form submission with use_resource
    #[cfg(feature = "web")]
    let future = use_resource(move || async move {
        if should_submit() {
            let (id, name, fields) = submit_data();
            Some(update_ping_format(guild_id, id, name, fields).await)
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    refetch_trigger.set(refetch_trigger() + 1);
                    show.set(false);
                    should_submit.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to update ping format: {}", err);
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
            error.set(Some("Ping format name is required".to_string()));
            return;
        }

        error.set(None);
        let field_data: Vec<(Option<i32>, String, i32, Option<String>)> = fields
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.name.trim().is_empty())
            .map(|(index, f)| {
                let default_val = if f.default_value.trim().is_empty() {
                    None
                } else {
                    Some(f.default_value.clone())
                };
                (f.id, f.name.clone(), index as i32, default_val)
            })
            .collect();

        let id = submit_data().0;
        submit_data.set((id, fields.name, field_data));
        should_submit.set(true);
    };

    let is_submitting = should_submit();

    rsx!(
        Modal {
            show,
            title: "Edit Ping Format".to_string(),
            prevent_close: is_submitting,
            form {
                class: "flex flex-col gap-4",
                onsubmit: on_submit,

                PingFormatFormFields {
                    form_fields,
                    is_submitting
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
fn PingFormatFormFields(mut form_fields: Signal<FormFieldsData>, is_submitting: bool) -> Element {
    let mut dragging_index = use_signal(|| None::<usize>);

    let add_field = move |_| {
        form_fields.write().fields.push(FieldData::default());
    };

    let on_drag_start = move |index: usize| {
        move |_evt: Event<DragData>| {
            dragging_index.set(Some(index));
        }
    };

    let on_drag_over = move |_evt: Event<DragData>| {
        _evt.prevent_default();
    };

    let on_drop = move |target_index: usize| {
        move |_evt: Event<DragData>| {
            _evt.prevent_default();
            if let Some(source_index) = dragging_index() {
                if source_index != target_index {
                    let mut fields = form_fields.write();
                    let item = fields.fields.remove(source_index);
                    fields.fields.insert(target_index, item);
                }
            }
            dragging_index.set(None);
        }
    };

    let on_drag_end = move |_evt: Event<DragData>| {
        dragging_index.set(None);
    };

    rsx!(
        // Name field
        div {
            class: "form-control w-full flex flex-col gap-2",
            label {
                class: "label",
                span { class: "label-text", "Ping Format Name" }
            }
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "e.g., Standard Ping, CTA Ping",
                value: "{form_fields().name}",
                disabled: is_submitting,
                oninput: move |evt| {
                    form_fields.write().name = evt.value();
                }
            }
        }

        // Fields section with add button
        div {
            class: "form-control w-full",
            div {
                class: "flex justify-between items-center mb-2",
                label {
                    class: "label",
                    span { class: "label-text font-semibold", "Fields" }
                }
                button {
                    r#type: "button",
                    class: "btn btn-sm btn-primary",
                    onclick: add_field,
                    disabled: is_submitting,
                    "＋ Add Field"
                }
            }

            SelectedItemsList {
                label: "".to_string(),
                empty_message: "No fields added. Click \"Add Field\" to create fields.".to_string(),
                is_empty: form_fields().fields.is_empty(),
                for (index, field) in form_fields().fields.iter().enumerate() {
                    {
                        let field_name = field.name.clone();
                        let field_default = field.default_value.clone();
                        let is_dragging = dragging_index() == Some(index);
                        rsx! {
                            div {
                                key: "{index}",
                                class: if is_dragging {
                                    "flex flex-col gap-2 p-3 bg-base-100 rounded-box opacity-50"
                                } else {
                                    "flex flex-col gap-2 p-3 bg-base-100 rounded-box"
                                },
                                ondragover: on_drag_over,
                                ondrop: on_drop(index),
                                div {
                                    class: "flex items-center gap-3",
                                    div {
                                        class: "cursor-move text-xl opacity-50 hover:opacity-100 select-none flex-shrink-0",
                                        title: "Drag to reorder",
                                        draggable: !is_submitting,
                                        ondragstart: on_drag_start(index),
                                        ondragend: on_drag_end,
                                        "⠿"
                                    }
                                    input {
                                        r#type: "text",
                                        class: "input input-bordered flex-1",
                                        placeholder: "Field name",
                                        value: "{field_name}",
                                        disabled: is_submitting,
                                        oninput: move |evt| {
                                            form_fields.write().fields[index].name = evt.value();
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn-sm btn-error btn-square flex-shrink-0",
                                        disabled: is_submitting,
                                        onclick: move |_| {
                                            form_fields.write().fields.remove(index);
                                        },
                                        "✕"
                                    }
                                }
                                input {
                                    r#type: "text",
                                    class: "input input-bordered input-sm w-full",
                                    placeholder: "Default value (optional)",
                                    value: "{field_default}",
                                    disabled: is_submitting,
                                    oninput: move |evt| {
                                        form_fields.write().fields[index].default_value = evt.value();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}
