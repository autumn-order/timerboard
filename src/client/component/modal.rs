use dioxus::prelude::*;

#[component]
pub fn Modal(
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
                class: "modal-box {class}",
                div {
                    h3 {
                        class: "font-bold text-lg mb-4",
                        "{title}"
                    }
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
