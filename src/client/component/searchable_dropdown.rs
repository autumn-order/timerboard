use dioxus::prelude::*;

/// Reusable searchable dropdown component that displays search results
#[component]
pub fn SearchableDropdown(
    /// Current search query signal
    mut search_query: Signal<String>,
    /// Placeholder text for the input
    placeholder: String,
    /// Display value when not searching (e.g., selected item name)
    display_value: Option<String>,
    /// Whether the component is disabled
    #[props(default = false)]
    disabled: bool,
    /// Whether the input is required
    #[props(default = false)]
    required: bool,
    /// Show empty state message
    #[props(default = "No items available".to_string())]
    empty_message: String,
    /// Show "not found" message
    #[props(default = "No matches found".to_string())]
    not_found_message: String,
    /// Whether there are items to display
    has_items: bool,
    /// Optional signal to control dropdown visibility from parent
    #[props(default = None)]
    show_dropdown_signal: Option<Signal<bool>>,
    /// Optional callback when input is focused
    #[props(default = None)]
    on_focus: Option<EventHandler<()>>,
    /// Dropdown content (rendered items)
    children: Element,
) -> Element {
    let local_show_dropdown = use_signal(|| false);
    let mut show_dropdown = show_dropdown_signal.unwrap_or(local_show_dropdown);

    rsx! {
        div {
            class: "relative",
            input {
                r#type: "text",
                class: "input input-bordered w-full",
                placeholder: "{placeholder}",
                value: if show_dropdown() {
                    "{search_query()}"
                } else if let Some(name) = display_value {
                    "{name}"
                } else {
                    ""
                },
                onfocus: move |_| {
                    show_dropdown.set(true);
                    search_query.set(String::new());
                    if let Some(handler) = on_focus {
                        handler.call(());
                    }
                },
                onclick: move |_| {
                    show_dropdown.set(true);
                },
                oninput: move |evt| {
                    search_query.set(evt.value());
                    show_dropdown.set(true);
                },
                disabled,
                required,
            }

            // Click outside to close dropdown
            if show_dropdown() {
                div {
                    class: "fixed inset-0 z-0",
                    onclick: move |_| {
                        show_dropdown.set(false);
                        search_query.set(String::new());
                    }
                }
            }

            if show_dropdown() {
                if has_items {
                    div {
                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg max-h-60 overflow-y-auto",
                        {children}
                    }
                } else {
                    div {
                        class: "absolute z-10 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg",
                        div {
                            class: "px-4 py-2 text-center opacity-50 text-sm",
                            if !search_query().is_empty() {
                                "{not_found_message}"
                            } else {
                                "{empty_message}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Dropdown item that can be clicked
#[component]
pub fn DropdownItem(
    /// Whether this item is currently selected
    #[props(default = false)]
    selected: bool,
    /// Callback when item is clicked
    on_select: EventHandler<()>,
    /// Item content
    children: Element,
) -> Element {
    rsx! {
        div {
            class: if selected {
                "px-4 py-2 cursor-pointer bg-primary text-primary-content hover:bg-primary-focus"
            } else {
                "px-4 py-2 cursor-pointer hover:bg-base-200"
            },
            onmousedown: move |evt| {
                evt.prevent_default();
                on_select.call(());
            },
            {children}
        }
    }
}

/// Reusable component for displaying a scrollable list of selected items
#[component]
pub fn SelectedItemsList(
    /// Label for the section
    label: String,
    /// Empty state message
    empty_message: String,
    /// Whether the list is empty
    is_empty: bool,
    /// List items content
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-2",
            label {
                class: "label",
                span { class: "label-text font-semibold", "{label}" }
            }
            div {
                class: if is_empty {
                    "space-y-2"
                } else {
                    "space-y-2 max-h-96 overflow-y-auto border border-base-300 rounded-lg p-2 bg-base-200"
                },
                if is_empty {
                    div {
                        class: "text-center py-8 opacity-50 text-sm",
                        "{empty_message}"
                    }
                } else {
                    {children}
                }
            }
        }
    }
}

/// Individual item in a selected items list
#[component]
pub fn SelectedItem(
    /// Whether the remove button is disabled
    #[props(default = false)]
    disabled: bool,
    /// Callback when remove button is clicked
    on_remove: EventHandler<()>,
    /// Item content
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 bg-base-100 rounded-lg",
            {children}
            button {
                r#type: "button",
                class: "btn btn-sm btn-error btn-square",
                disabled,
                onclick: move |_| {
                    on_remove.call(());
                },
                "✕"
            }
        }
    }
}
