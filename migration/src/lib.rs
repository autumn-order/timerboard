pub use sea_orm_migration::prelude::*;

mod m20251210_000001_create_user_table;
mod m20251211_000002_create_discord_guild_table;
mod m20251211_000003_create_discord_guild_role_table;
mod m20251211_000004_create_user_discord_guild_table;
mod m20251211_000005_create_user_discord_guild_role_table;
mod m20251211_000006_create_discord_guild_channel_table;
mod m20251212_000007_create_ping_format_table;
mod m20251212_000008_create_ping_format_fields_table;
mod m20251212_000009_create_fleet_category_table;
mod m20251212_000010_create_fleet_category_access_role_table;
mod m20251212_000011_create_fleet_category_ping_role_table;
mod m20251212_000012_create_fleet_category_channel_table;
mod m20251212_000013_create_fleet_table;
mod m20251212_000014_create_fleet_field_value_table;
mod m20251212_000015_create_fleet_message_table;
mod m20251216_000016_create_channel_fleet_list;
mod m20251223_000017_create_ping_group;
mod m20251223_000018_add_ping_group_to_fleet_category;
mod m20251225_000019_add_value_type_to_ping_format_fields;
mod m20251225_000020_create_ping_format_field_values;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251210_000001_create_user_table::Migration),
            Box::new(m20251211_000002_create_discord_guild_table::Migration),
            Box::new(m20251211_000003_create_discord_guild_role_table::Migration),
            Box::new(m20251211_000004_create_user_discord_guild_table::Migration),
            Box::new(m20251211_000005_create_user_discord_guild_role_table::Migration),
            Box::new(m20251211_000006_create_discord_guild_channel_table::Migration),
            Box::new(m20251212_000007_create_ping_format_table::Migration),
            Box::new(m20251212_000008_create_ping_format_fields_table::Migration),
            Box::new(m20251212_000009_create_fleet_category_table::Migration),
            Box::new(m20251212_000010_create_fleet_category_access_role_table::Migration),
            Box::new(m20251212_000011_create_fleet_category_ping_role_table::Migration),
            Box::new(m20251212_000012_create_fleet_category_channel_table::Migration),
            Box::new(m20251212_000013_create_fleet_table::Migration),
            Box::new(m20251212_000014_create_fleet_field_value_table::Migration),
            Box::new(m20251212_000015_create_fleet_message_table::Migration),
            Box::new(m20251216_000016_create_channel_fleet_list::Migration),
            Box::new(m20251223_000017_create_ping_group::Migration),
            Box::new(m20251223_000018_add_ping_group_to_fleet_category::Migration),
            Box::new(m20251225_000019_add_value_type_to_ping_format_fields::Migration),
            Box::new(m20251225_000020_create_ping_format_field_values::Migration),
        ]
    }
}
