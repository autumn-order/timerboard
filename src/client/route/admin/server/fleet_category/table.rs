use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::{
    client::{
        component::{ConfirmationModal, Pagination, PaginationData},
        route::admin::server::FleetCategoriesCache,
    },
    model::fleet::PaginatedFleetCategoriesDto,
};

use super::modals::EditCategoryModal;

#[cfg(feature = "web")]
use crate::client::api::fleet_category::delete_fleet_category;

#[component]
pub fn FleetCategoriesTable(
    data: PaginatedFleetCategoriesDto,
    guild_id: u64,
    mut cache: Signal<FleetCategoriesCache>,
    mut refetch_trigger: Signal<u32>,
) -> Element {
    let mut sorted_categories = data.categories.clone();
    sorted_categories.sort_by_key(|c| c.id);

    let mut show_delete_modal = use_signal(|| false);
    let mut category_to_delete = use_signal(|| None::<(i32, String)>);
    let mut is_deleting = use_signal(|| false);

    let mut show_edit_modal = use_signal(|| false);
    let mut category_id_to_edit = use_signal(|| None::<i32>);

    // Handle deletion with use_resource
    #[cfg(feature = "web")]
    let delete_future = use_resource(move || async move {
        if is_deleting() {
            if let Some((id, _)) = category_to_delete() {
                Some(delete_fleet_category(guild_id, id).await)
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
                    tracing::error!("Failed to delete category: {}", err);
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
                        th { "Ping Format" }
                        th { class: "text-center", "Upcoming Fleets" }
                        th { class: "text-center", "All-time Total" }
                        th { class: "text-center", "Configured Roles" }
                        th {
                            class: "text-right",
                            "Actions"
                        }
                    }
                }
                tbody {
                    for category in &sorted_categories {
                        {
                            let category_id = category.id;
                            let category_name = category.name.clone();
                            let category_name_for_delete = category_name.clone();
                            rsx! {
                                tr {
                                    td { "{category.name}" }
                                    td { "{category.ping_format_name}" }
                                    td { class: "text-center", "0" }
                                    td { class: "text-center", "0" }
                                    td { class: "text-center", "0" }
                                    td {
                                        div {
                                            class: "flex gap-2 justify-end",
                                            button {
                                                class: "btn btn-sm btn-primary",
                                                onclick: move |_| {
                                                    category_id_to_edit.set(Some(category_id));
                                                    show_edit_modal.set(true);
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-sm btn-error",
                                                onclick: move |_| {
                                                    category_to_delete.set(Some((category_id, category_name_for_delete.clone())));
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

        // Delete Confirmation Modal
        ConfirmationModal {
            show: show_delete_modal,
            title: "Delete Fleet Category".to_string(),
            message: rsx!(
                if let Some((_, name)) = category_to_delete() {
                    p {
                        class: "py-4",
                        "Are you sure you want to delete the category "
                        span { class: "font-bold", "\"{name}\"" }
                        "? This action cannot be undone."
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

        // Edit Category Modal
        EditCategoryModal {
            guild_id,
            show: show_edit_modal,
            category_id: category_id_to_edit,
            refetch_trigger
        }
    )
}

#[component]
pub fn FleetCategoryPagination(
    mut page: Signal<u64>,
    mut per_page: Signal<u64>,
    pagination_data: PaginatedFleetCategoriesDto,
    mut cache: Signal<FleetCategoriesCache>,
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
