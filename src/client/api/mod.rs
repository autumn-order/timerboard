#[cfg(feature = "web")]
pub mod helper;

#[cfg(feature = "web")]
pub mod discord_guild;

#[cfg(feature = "web")]
pub mod fleet_category;

#[cfg(feature = "web")]
pub use discord_guild::{get_all_discord_guilds, get_discord_guild_by_id};

#[cfg(feature = "web")]
pub use fleet_category::{
    create_fleet_category, delete_fleet_category, get_fleet_categories, update_fleet_category,
};
