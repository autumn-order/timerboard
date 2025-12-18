use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{ConfirmationModal, Pagination, PaginationData},
        route::admin::server::PingFormatsCache,
    },
    model::ping_format::PaginatedPingFormatsDto,
};

use super::modal::EditPingFormatModal;

#[cfg(feature = "web")]
use crate::client::api::ping_format::delete_ping_format;

#[component]
pub fn PingFormatsTable(
    data: PaginatedPingFormatsDto,
    guild_id: u64,
    mut cache: Signal<PingFormatsCache>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut sorted_formats = data.ping_formats.clone();
    sorted_formats.sort_by_key(|f| f.id);

    let mut show_delete_modal = use_signal(|| false);
    let mut format_to_delete = use_signal(|| None::<(i32, String, u64)>);
    let mut is_deleting = use_signal(|| false);

    let mut show_edit_modal = use_signal(|| false);
    let mut format_to_edit = use_signal(|| None::<crate::model::ping_format::PingFormatDto>);

    // Handle deletion with use_resource
    #[cfg(feature = "web")]
    let delete_future = use_resource(move || async move {
        if is_deleting() {
            if let Some((id, _, _)) = format_to_delete() {
                Some(delete_ping_format(guild_id, id).await)
            } else {
                None
            }
        } else {
            None
        }
    });

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = delete_future.read_unchecked().as_ref() {
            match result {
                Ok(_) => {
                    // Trigger refetch
                    refetch_trigger.set(refetch_trigger() + 1);
                    // Close modal (data persists for smooth animation)
                    show_delete_modal.set(false);
                    is_deleting.set(false);
                }
                Err(err) => {
                    tracing::error!("Failed to delete ping format: {}", err);
                    is_deleting.set(false);
                }
            }
        }
    });

    rsx!(
        div {
            class: "overflow-x-auto",
            table {
                class: "table table-zebra w-full",
                thead {
                    tr {
                        th { "Name" }
                        th { "Fleet Categories" }
                        th { "Fields" }
                        th {
                            class: "text-right",
                            "Actions"
                        }
                    }
                }
                tbody {
                    for format in &sorted_formats {
                        {
                            let format_id = format.id;
                            let format_name = format.name.clone();
                            let format_clone_for_edit = format.clone();
                            let format_name_for_delete = format_name.clone();
                            let fleet_category_count = format.fleet_category_count;
                            let fleet_category_names = format.fleet_category_names.clone();
                            let field_names: Vec<String> = format.fields.iter().map(|f| f.name.clone()).collect();
                            let field_display = field_names.join(", ");
                            let category_display = fleet_category_names.join(", ");
                            rsx! {
                                tr {
                                    td { "{format.name}" }
                                    td {
                                        class: "max-w-xs break-words",
                                        if fleet_category_names.is_empty() {
                                            span { class: "opacity-50", "No categories" }
                                        } else {
                                            span { "{category_display}" }
                                        }
                                    }
                                    td {
                                        if field_names.is_empty() {
                                            span { class: "opacity-50", "No fields" }
                                        } else {
                                            span { "{field_display}" }
                                        }
                                    }
                                    td {
                                        div {
                                            class: "flex gap-2 justify-end",
                                            button {
                                                class: "btn btn-sm btn-primary",
                                                onclick: move |_| {
                                                    format_to_edit.set(Some(format_clone_for_edit.clone()));
                                                    show_edit_modal.set(true);
                                                },
                                                "Edit"
                                            }
                                            if fleet_category_count > 0 {
                                                div {
                                                    class: "tooltip tooltip-left",
                                                    "data-tip": format!("Cannot delete: {} fleet {} using this format",
                                                        fleet_category_count,
                                                        if fleet_category_count == 1 { "category is" } else { "categories are" }),
                                                    button {
                                                        class: "btn btn-sm btn-disabled",
                                                        disabled: true,
                                                        "Delete"
                                                    }
                                                }
                                            } else {
                                                button {
                                                    class: "btn btn-sm btn-error",
                                                    onclick: move |_| {
                                                        format_to_delete.set(Some((format_id, format_name_for_delete.clone(), fleet_category_count)));
                                                        show_delete_modal.set(true);
                                                    },
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Delete Confirmation Modal
        ConfirmationModal {
            show: show_delete_modal,
            title: "Delete Ping Format".to_string(),
            message: rsx!(
                if let Some((_, name, _)) = format_to_delete() {
                    div {
                        class: "py-4",
                        p {
                            "Are you sure you want to delete the ping format "
                            span { class: "font-bold", "\"{name}\"" }
                            "?"
                        }
                        p {
                            class: "mt-4",
                            "This action cannot be undone."
                        }
                    }
                }
            ),
            confirm_text: "Delete".to_string(),
            confirm_class: "btn-error".to_string(),
            is_processing: is_deleting(),
            processing_text: "Deleting...".to_string(),
            on_confirm: move |_| {
                is_deleting.set(true);
            },
        }

        // Edit Ping Format Modal
        EditPingFormatModal {
            guild_id,
            show: show_edit_modal,
            format_to_edit,
            refetch_trigger
        }
    )
}

#[component]
pub fn PingFormatPagination(
    mut page: Signal<u64>,
    mut per_page: Signal<u64>,
    pagination_data: PaginatedPingFormatsDto,
    mut cache: Signal<PingFormatsCache>,
) -> Element {
    let data = PaginationData {
        page: pagination_data.page,
        per_page: pagination_data.per_page,
        total: pagination_data.total,
        total_pages: pagination_data.total_pages,
    };

    rsx!(Pagination {
        page,
        per_page,
        data,
        on_page_change: move |new_page| {
            cache.write().page = new_page;
        },
        on_per_page_change: move |new_per_page| {
            cache.write().per_page = new_per_page;
            cache.write().page = 0;
        },
    })
}
