use dioxus::prelude::*;

use super::Modal;

#[derive(Clone, PartialEq)]
pub struct PaginationData {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

#[component]
pub fn Pagination(
    page: Signal<u64>,
    per_page: Signal<u64>,
    data: PaginationData,
    on_page_change: EventHandler<u64>,
    on_per_page_change: EventHandler<u64>,
) -> Element {
    let mut show_page_jump = use_signal(|| false);
    let mut jump_page_input = use_signal(String::new);

    rsx!(
        div {
            class: "flex flex-col sm:flex-row justify-between items-center mt-4 gap-4",
            // Per-page selector
            div {
                class: "flex items-center gap-2 text-sm",
                span { "Show" }
                select {
                    class: "select select-bordered select-sm",
                    value: "{per_page()}",
                    onchange: move |evt| {
                        if let Ok(value) = evt.value().parse::<u64>() {
                            per_page.set(value);
                            page.set(0); // Reset to first page
                            on_per_page_change.call(value);
                        }
                    },
                    option { value: "5", "5" }
                    option { value: "10", "10" }
                    option { value: "25", "25" }
                    option { value: "50", "50" }
                    option { value: "100", "100" }
                }
                span { "entries" }
            }

            // Pagination info and buttons
            div {
                class: "flex flex-col sm:flex-row items-center gap-2 sm:gap-4",
                span {
                    class: "text-xs sm:text-sm opacity-70 whitespace-nowrap",
                    "Showing {(data.page * data.per_page) + 1} to {((data.page + 1) * data.per_page).min(data.total)} of {data.total}"
                }
                div {
                    class: "join",
                    button {
                        class: "join-item btn btn-xs sm:btn-sm",
                        disabled: data.page == 0,
                        onclick: move |_| {
                            if page() > 0 {
                                let new_page = page() - 1;
                                page.set(new_page);
                                on_page_change.call(new_page);
                            }
                        },
                        "«"
                    }
                    button {
                        class: "join-item btn btn-xs sm:btn-sm",
                        onclick: move |_| {
                            jump_page_input.set((data.page + 1).to_string());
                            show_page_jump.set(true);
                        },
                        "Page {data.page + 1} of {data.total_pages}"
                    }
                    button {
                        class: "join-item btn btn-xs sm:btn-sm",
                        disabled: data.page >= data.total_pages - 1,
                        onclick: move |_| {
                            if page() < data.total_pages - 1 {
                                let new_page = page() + 1;
                                page.set(new_page);
                                on_page_change.call(new_page);
                            }
                        },
                        "»"
                    }
                }
            }
        }

        // Page Jump Modal
        Modal {
            show: show_page_jump,
            title: "Jump to Page".to_string(),
            prevent_close: false,
            form {
                onsubmit: move |evt| {
                    evt.prevent_default();
                    if let Ok(target_page) = jump_page_input().parse::<u64>() {
                        if target_page > 0 && target_page <= data.total_pages {
                            let new_page = target_page - 1; // Convert to 0-indexed
                            page.set(new_page);
                            on_page_change.call(new_page);
                            show_page_jump.set(false);
                        }
                    }
                },
                div {
                    class: "form-control w-full flex flex-col gap-3",
                    label {
                        class: "label",
                        span {
                            class: "label-text",
                            "Page number (1-{data.total_pages})"
                        }
                    }
                    input {
                        r#type: "number",
                        class: "input input-bordered w-full",
                        min: "1",
                        max: "{data.total_pages}",
                        value: "{jump_page_input()}",
                        oninput: move |evt| jump_page_input.set(evt.value()),
                        autofocus: true,
                    }
                }
                div {
                    class: "modal-action",
                    button {
                        r#type: "button",
                        class: "btn",
                        onclick: move |_| show_page_jump.set(false),
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        "Jump"
                    }
                }
            }
        }
    )
}
