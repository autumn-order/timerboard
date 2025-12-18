//! Service layer for business logic and orchestration.
//!
//! This module contains the service layer of the application, which sits between the
//! controller (API) layer and the data (repository) layer. Services are responsible for:
//!
//! - **Business Logic**: Implementing core business rules and validation
//! - **Orchestration**: Coordinating multiple repository calls and external services
//! - **Domain Models**: Working with domain models rather than DTOs or entity models
//! - **Transaction Management**: Handling complex multi-step operations

pub mod admin;
pub mod auth;
pub mod category;
pub mod discord;
pub mod fleet;
pub mod fleet_notification;
pub mod ping_format;
pub mod user;
