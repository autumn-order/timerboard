use dioxus::prelude::*;

/// Larger modal that goes fullscreen on smaller device sizes
#[component]
pub fn FullScreenModal(
    show: Signal<bool>,
    title: String,
    prevent_close: bool,
    class: Option<&'static str>,
    children: Element,
) -> Element {
    let class: &str = class.unwrap_or_default();
    // Focus modal when it opens
    #[cfg(feature = "web")]
    use_effect(move || {
        if show() {
            document::eval(r#"document.querySelector('.modal-open')?.focus()"#);
        }
    });

    rsx!(
        div {
            class: if show() { "modal modal-open" } else { "modal" },
            tabindex: "-1",
            onkeydown: move |evt| {
                if evt.key() == Key::Escape && !prevent_close {
                    show.set(false);
                }
            },
            div {
                class: "modal-box {class} w-full h-full border border-base-300 max-w-none max-h-none sm:w-11/12 sm:max-w-5xl sm:h-auto sm:max-h-[90vh] m-0 sm:m-auto rounded-none sm:rounded-box",
                // Header with title and close button
                div {
                    class: "flex justify-between items-center mb-4",
                    h3 {
                        class: "font-bold text-lg",
                        "{title}"
                    }
                    if !prevent_close {
                        button {
                            class: "btn btn-sm btn-circle btn-ghost",
                            onclick: move |_| show.set(false),
                            "✕"
                        }
                    }
                }
                // Content
                div {
                    {children}
                }
            }
            div {
                class: "modal-backdrop",
                onclick: move |_| {
                    if !prevent_close {
                        show.set(false);
                    }
                },
            }
        }
    )
}
