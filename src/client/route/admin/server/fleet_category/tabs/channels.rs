use dioxus::prelude::*;

use crate::client::component::{DropdownItem, SearchableDropdown, SelectedItem, SelectedItemsList};
use crate::model::discord::DiscordGuildChannelDto;

use super::super::types::{ChannelData, FormFieldsData};

#[cfg(feature = "web")]
use crate::client::api::discord::get_discord_guild_channels;

#[component]
pub fn ChannelsTab(
    guild_id: u64,
    mut form_fields: Signal<FormFieldsData>,
    is_submitting: bool,
) -> Element {
    let mut available_channels = use_signal(|| Vec::<DiscordGuildChannelDto>::new());
    let mut channel_search_query = use_signal(|| String::new());
    let mut channel_dropdown_open = use_signal(|| false);

    // Fetch channels when component mounts
    #[cfg(feature = "web")]
    let channels_future =
        use_resource(
            move || async move { get_discord_guild_channels(guild_id, 0, 1000).await.ok() },
        );

    #[cfg(feature = "web")]
    use_effect(move || {
        if let Some(Some(result)) = channels_future.read_unchecked().as_ref() {
            available_channels.set(result.channels.clone());
        }
    });

    // Filter available channels by search query and exclude already added channels
    let filtered_channels = use_memo(move || {
        let channels = available_channels();
        let query = channel_search_query().to_lowercase();
        let channel_ids: Vec<i64> = form_fields().channels.iter().map(|c| c.id).collect();

        let mut filtered: Vec<_> = channels
            .into_iter()
            .filter(|c| !channel_ids.contains(&c.channel_id))
            .filter(|c| {
                if query.is_empty() {
                    true
                } else {
                    c.name.to_lowercase().contains(&query)
                }
            })
            .collect();

        // Sort by position ascending (lower position first)
        filtered.sort_by_key(|c| c.position);
        filtered
    });

    // Sort channels by position
    let sorted_channels = use_memo(move || {
        let mut channels = form_fields().channels.clone();
        let channel_positions: std::collections::HashMap<i64, i32> = available_channels()
            .iter()
            .map(|c| (c.channel_id, c.position))
            .collect();

        channels.sort_by_key(|c| channel_positions.get(&c.id).copied().unwrap_or(i32::MAX));
        channels
    });

    rsx! {
        div {
            class: "space-y-4",
            // Search and add channel
            div {
                class: "form-control flex flex-col gap-2",
                label {
                    class: "label",
                    span { class: "label-text", "Add Channel" }
                }
                SearchableDropdown {
                    search_query: channel_search_query,
                    placeholder: "Search channels...".to_string(),
                    display_value: None,
                    disabled: is_submitting,
                    has_items: !filtered_channels().is_empty(),
                    show_dropdown_signal: Some(channel_dropdown_open),
                    for channel in filtered_channels() {
                        {
                            let channel_name = channel.name.clone();
                            rsx! {
                                DropdownItem {
                                    key: "{channel.channel_id}",
                                    on_select: move |_| {
                                        let new_channel = ChannelData {
                                            id: channel.channel_id,
                                            name: channel.name.clone(),
                                        };
                                        form_fields.write().channels.push(new_channel);
                                        channel_search_query.set(String::new());
                                        channel_dropdown_open.set(false);
                                    },
                                    "# {channel_name}"
                                }
                            }
                        }
                    }
                }
            }

            // List of channels with scrollable container
            SelectedItemsList {
                label: "Configured Channels".to_string(),
                empty_message: "No channels configured. Add channels where fleet notifications will be sent.".to_string(),
                is_empty: sorted_channels().is_empty(),
                for channel in sorted_channels() {
                    {
                        let channel_id = channel.id;
                        let channel_name = channel.name.clone();
                        // Find the actual index in form_fields
                        let actual_index = form_fields().channels.iter().position(|c| c.id == channel_id).unwrap_or(0);
                        rsx! {
                            SelectedItem {
                                key: "{channel_id}",
                                disabled: is_submitting,
                                on_remove: move |_| {
                                    form_fields.write().channels.remove(actual_index);
                                },
                                div {
                                    class: "flex-1 font-medium",
                                    "# {channel_name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
