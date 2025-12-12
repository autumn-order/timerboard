use dioxus::prelude::*;

use super::Modal;

#[component]
pub fn ConfirmationModal(
    show: Signal<bool>,
    title: String,
    message: Element,
    confirm_text: String,
    confirm_class: String,
    is_processing: bool,
    processing_text: String,
    on_confirm: EventHandler<()>,
) -> Element {
    rsx!(
        Modal {
            show,
            title,
            prevent_close: is_processing,
            {message}
            div {
                class: "modal-action",
                button {
                    r#type: "button",
                    class: "btn",
                    onclick: move |_| {
                        show.set(false);
                    },
                    disabled: is_processing,
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "btn {confirm_class}",
                    onclick: move |_| {
                        on_confirm.call(());
                    },
                    disabled: is_processing,
                    if is_processing {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "{processing_text}"
                    } else {
                        "{confirm_text}"
                    }
                }
            }
        }
    )
}
