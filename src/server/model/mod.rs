//! Server-side domain models and parameter types.
//!
//! This module contains domain models used throughout the service layer, representing
//! business entities and operation parameters. Domain models are converted from entity
//! models at the repository boundary and transformed to DTOs at the controller boundary.
//! They provide type-safe representations with business logic separated from database
//! and API concerns.

pub mod category;
pub mod channel_fleet_list;
pub mod discord;
pub mod fleet;
pub mod fleet_message;
pub mod ping_format;
pub mod user;
