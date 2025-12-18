//! Discord bot integration for automated guild and member management.
//!
//! This module provides Discord bot functionality for the application, enabling
//! automated interactions with Discord guilds and members. The bot handles events
//! such as guild joins, member updates, and other Discord-related activities that
//! require real-time event processing.
//!
//! The bot is initialized during server startup and runs in a separate tokio task
//! to avoid blocking the main HTTP server. The bot's HTTP client is shared with
//! other services (such as fleet notifications) to send messages and embeds without
//! maintaining multiple connections to Discord.
//!
//! # Gateway Intents
//!
//! The bot requires the following gateway intents:
//! - `GUILDS` - Receive events about guild creation, updates, and deletion
//! - `GUILD_MESSAGES` - Receive events about messages in guilds
//! - `GUILD_MEMBERS` - Receive events about guild member changes (privileged intent)
//!
//! Note: `GUILD_MEMBERS` is a privileged intent and must be explicitly enabled
//! in the Discord Developer Portal for the bot application.

pub mod handler;
pub mod start;
