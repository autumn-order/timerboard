//! Database repository layer for all domain entities.
//!
//! This module contains repository structs that handle database operations (CRUD) for each
//! domain in the application. Repositories use SeaORM entity models internally and return
//! parameter models to maintain separation between the data layer and business logic layer.
//! All database queries, inserts, updates, and deletes are performed through these repositories.

pub mod category;
pub mod channel_fleet_list;
pub mod discord;
pub mod fleet;
pub mod fleet_message;
pub mod ping_format;
pub mod ping_group;
pub mod user;
pub mod user_category_permission;

#[cfg(test)]
mod test;
