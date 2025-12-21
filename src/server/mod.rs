//! Server-side API backend and business logic.
//!
//! This module contains the complete backend implementation for the application,
//! including API endpoints, business logic, data access, and infrastructure services.
//! The backend uses Axum as the web framework, SeaORM for database operations, and
//! Serenity for Discord bot integration.
//!
//! # Architecture
//!
//! The server follows a layered architecture with clear separation of concerns:
//!
//! - **Controller Layer** (`controller/`) - HTTP request handlers, access control, and DTO conversion
//! - **Service Layer** (`service/`) - Business logic orchestration between controllers and data layer
//! - **Data Layer** (`data/`) - Database operations and entity-to-domain model conversion
//! - **Model Layer** (`model/`) - Domain models and operation-specific parameter types
//! - **Error Layer** (`error/`) - Application error types and HTTP response mapping
//! - **Middleware** (`middleware/`) - Request/response processing and authentication guards
//!
//! # Infrastructure
//!
//! Supporting modules provide application infrastructure:
//!
//! - **Configuration** (`config`) - Environment-based application configuration
//! - **State** (`state`) - Shared application state (DB, HTTP clients, etc.)
//! - **Startup** (`startup`) - Initialization of database, sessions, and services
//! - **Router** (`router`) - Axum route configuration and API documentation
//! - **Scheduler** (`scheduler/`) - Cron jobs for automated tasks (fleet notifications, etc.)
//! - **Bot** (`bot/`) - Discord bot event handlers and integration
//!
//! # Request Flow
//!
//! A typical request flows through these layers:
//!
//! 1. **Router** receives HTTP request and routes to appropriate controller
//! 2. **Middleware** processes authentication and session management
//! 3. **Controller** validates access, converts DTOs to params, calls service
//! 4. **Service** executes business logic, orchestrates data operations
//! 5. **Data** queries database, converts entities to domain models
//! 6. **Service** returns domain model to controller
//! 7. **Controller** converts domain model to DTO, returns HTTP response
//!
//! # Feature Gates
//!
//! This module is only available with the `server` feature flag enabled.

pub mod bot;
pub mod config;
pub mod controller;
pub mod data;
pub mod error;
pub mod middleware;
pub mod model;
pub mod router;
pub mod scheduler;
pub mod service;
pub mod startup;
pub mod state;
pub mod util;
