//! Ready event handler for bot initialization.
//!
//! This module handles the `ready` event which is fired when the bot successfully
//! connects to Discord's gateway and completes the initial handshake. This is the
//! first event received after authentication and indicates the bot is ready to
//! process other events.
//!
//! The ready handler is used to:
//! - Log connection information
//! - Perform any one-time initialization tasks

use dioxus_logger::tracing;
use serenity::all::{Context, Ready};

/// Handles the ready event when the bot connects to Discord.
///
/// This event fires once per bot connection after successful authentication and
/// initial gateway handshake. It indicates the bot is now connected and ready to
/// receive and process other events.
///
/// # Arguments
/// - `ctx` - Discord context for setting activity status
/// - `ready` - Ready event data containing bot user information
pub async fn handle_ready(_ctx: Context, ready: Ready) {
    tracing::info!("{} is connected to Discord", ready.user.name);
}
