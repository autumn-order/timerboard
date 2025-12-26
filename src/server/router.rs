//! HTTP routing and OpenAPI documentation configuration.
//!
//! This module defines the application's HTTP routes and generates OpenAPI documentation
//! using utoipa. All API endpoints are registered here with their OpenAPI specifications,
//! and Swagger UI is configured to provide interactive API documentation at `/api/docs`.

use axum::{
    http::{header, Method},
    Router,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    model::{
        api::{ErrorDto, SuccessDto},
        category::{
            CreateFleetCategoryDto, FleetCategoryAccessRoleDto, FleetCategoryChannelDto,
            FleetCategoryDetailsDto, FleetCategoryDto, FleetCategoryListItemDto,
            FleetCategoryPingRoleDto, PaginatedFleetCategoriesDto, UpdateFleetCategoryDto,
        },
        discord::{
            DiscordGuildChannelDto, DiscordGuildDto, DiscordGuildMemberDto, DiscordGuildRoleDto,
            PaginatedDiscordGuildChannelsDto, PaginatedDiscordGuildRolesDto,
        },
        fleet::{CreateFleetDto, FleetDto, FleetListItemDto, PaginatedFleetsDto, UpdateFleetDto},
        ping_format::{
            CreatePingFormatDto, CreatePingFormatFieldDto, PaginatedPingFormatsDto, PingFormatDto,
            PingFormatFieldDto, UpdatePingFormatDto, UpdatePingFormatFieldDto,
        },
        user::{PaginatedUsersDto, UserDto},
    },
    server::{config::Config, controller, error::AppError, state::AppState},
};

/// Creates the main router with CORS configuration and OpenAPI documentation.
///
/// Configures CORS based on the allowed origins from the application config.
/// CORS is applied to all API routes with credentials support enabled for
/// session-based authentication.
///
/// Configures rate limiting using tower-governor with IP-based key extraction.
/// Rate limits are set to 100 requests per minute per IP address to prevent abuse.
/// The rate limiter uses a GCRA (Generic Cell Rate Algorithm) for smooth rate limiting.
///
/// Constructs an Axum router with all authentication, user management, admin, fleet, category,
/// and discord endpoints registered. Each endpoint is annotated with OpenAPI specifications via
/// utoipa, which are collected into a unified OpenAPI document. The router includes Swagger UI
/// at `/api/docs` for interactive API exploration and testing (debug builds only).
///
/// # Registered Endpoints
///
/// ## Authentication (`/api/auth`)
/// - `GET /api/auth/login` - Initiate Discord OAuth authentication
/// - `GET /api/auth/callback` - OAuth callback handler
/// - `GET /api/auth/logout` - Logout current user
/// - `GET /api/auth/user` - Get current user information
///
/// ## User (`/api/user`)
/// - `GET /api/user/guilds` - Get guilds available to current user
/// - `GET /api/user/guilds/{guild_id}/manageable-categories` - Get manageable categories
///
/// ## Admin (`/api/admin`)
/// - `GET /api/admin/bot/add` - Add bot to Discord server
/// - `GET /api/admin/users` - Get all users (paginated)
/// - `GET /api/admin/admins` - Get all admins
/// - `POST /api/admin/admins/{user_id}` - Add admin
/// - `DELETE /api/admin/admins/{user_id}` - Remove admin
/// - `GET /api/admin/servers` - Get all Discord guilds
/// - `GET /api/admin/servers/{guild_id}` - Get Discord guild by ID
/// - `GET /api/admin/servers/{guild_id}/roles` - Get guild roles
/// - `GET /api/admin/servers/{guild_id}/channels` - Get guild channels
///
/// ## Categories (`/api/admin/servers/{guild_id}/categories`)
/// - `GET /api/admin/servers/{guild_id}/categories` - Get all categories
/// - `POST /api/admin/servers/{guild_id}/categories` - Create category
/// - `GET /api/admin/servers/{guild_id}/categories/{category_id}` - Get category by ID
/// - `PUT /api/admin/servers/{guild_id}/categories/{category_id}` - Update category
/// - `DELETE /api/admin/servers/{guild_id}/categories/{category_id}` - Delete category
///
/// ## Ping Formats (`/api/admin/servers/{guild_id}/formats`)
/// - `GET /api/admin/servers/{guild_id}/formats` - Get all ping formats
/// - `POST /api/admin/servers/{guild_id}/formats` - Create ping format
/// - `PUT /api/admin/servers/{guild_id}/formats/{format_id}` - Update ping format
/// - `DELETE /api/admin/servers/{guild_id}/formats/{format_id}` - Delete ping format
/// - `GET /api/admin/servers/{guild_id}/formats/{format_id}/categories` - Get categories by format
///
/// ## Fleets (`/api/guilds/{guild_id}`)
/// - `GET /api/guilds/{guild_id}/members` - Get guild members
/// - `GET /api/guilds/{guild_id}/categories/{category_id}/details` - Get category details
/// - `GET /api/guilds/{guild_id}/fleets` - Get all fleets
/// - `POST /api/guilds/{guild_id}/fleets` - Create fleet
/// - `GET /api/guilds/{guild_id}/fleets/{fleet_id}` - Get fleet by ID
/// - `PUT /api/guilds/{guild_id}/fleets/{fleet_id}` - Update fleet
/// - `DELETE /api/guilds/{guild_id}/fleets/{fleet_id}` - Delete fleet
///
/// # OpenAPI Documentation
/// The OpenAPI specification is available at `/api/docs/openapi.json` and includes:
/// - Endpoint paths and HTTP methods
/// - Request/response schemas
/// - Authentication requirements
/// - Error responses
///
/// # Swagger UI
/// Interactive API documentation is served at `/api/docs` **only in debug builds**.
/// In release builds, the Swagger UI endpoint is not available for security reasons.
/// Allowing developers to:
/// - Browse available endpoints
/// - View request/response schemas
/// - Test endpoints directly from the browser
/// - Download the OpenAPI specification
///
/// # Rate Limiting
/// Rate limiting is configured globally for all routes:
/// - **10 requests per second** per IP address
/// - **Burst of 20** allows brief spikes in traffic
/// - Uses `SmartIpKeyExtractor` to handle X-Forwarded-For and X-Real-IP headers
/// - Returns 429 Too Many Requests when limit is exceeded
/// - Response headers include rate limit information (X-RateLimit-*)
///
/// # Arguments
/// - `config` - Application configuration containing CORS origins
///
/// # Returns
/// - `Ok(Router<AppState>)` - Configured router with CORS and rate limiting layers applied
/// - `Err(AppError::InternalError)` - If app_url fails to parse or rate limiter configuration fails
pub fn router(config: &Config) -> Result<Router<AppState>, AppError> {
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "Timerboard API",
            description = "API for managing EVE Online fleet timers and Discord integration"
        ),
        tags(
            (name = controller::auth::AUTH_TAG, description = "Authentication endpoints"),
            (name = controller::user::USER_TAG, description = "User endpoints"),
            (name = controller::admin::ADMIN_TAG, description = "Admin endpoints"),
            (name = controller::category::CATEGORY_TAG, description = "Fleet category endpoints"),
            (name = controller::ping_format::PING_FORMAT_TAG, description = "Ping format endpoints"),
            (name = controller::fleet::FLEET_TAG, description = "Fleet endpoints"),
            (name = controller::discord::DISCORD_TAG, description = "Discord endpoints"),
        ),
        components(
            schemas(
                ErrorDto,
                SuccessDto,
                UserDto,
                PaginatedUsersDto,
                DiscordGuildDto,
                DiscordGuildMemberDto,
                DiscordGuildRoleDto,
                DiscordGuildChannelDto,
                PaginatedDiscordGuildRolesDto,
                PaginatedDiscordGuildChannelsDto,
                FleetCategoryDto,
                FleetCategoryListItemDto,
                PaginatedFleetCategoriesDto,
                FleetCategoryDetailsDto,
                FleetCategoryAccessRoleDto,
                FleetCategoryPingRoleDto,
                FleetCategoryChannelDto,
                CreateFleetCategoryDto,
                UpdateFleetCategoryDto,
                PingFormatDto,
                PingFormatFieldDto,
                CreatePingFormatDto,
                CreatePingFormatFieldDto,
                UpdatePingFormatDto,
                UpdatePingFormatFieldDto,
                PaginatedPingFormatsDto,
                FleetDto,
                FleetListItemDto,
                PaginatedFleetsDto,
                CreateFleetDto,
                UpdateFleetDto,
            )
        )
    )]
    struct ApiDoc;

    // Auth routes
    let auth_routes = OpenApiRouter::new()
        .routes(routes!(controller::auth::login))
        .routes(routes!(controller::auth::callback))
        .routes(routes!(controller::auth::logout))
        .routes(routes!(controller::auth::get_user));

    // User routes
    let user_routes = OpenApiRouter::new()
        .routes(routes!(controller::user::get_user_guilds))
        .routes(routes!(controller::user::get_user_manageable_categories));

    // Admin routes
    let admin_routes = OpenApiRouter::new()
        .routes(routes!(controller::admin::add_bot))
        .routes(routes!(controller::admin::get_all_users))
        .routes(routes!(controller::admin::get_all_admins))
        .routes(routes!(controller::admin::add_admin))
        .routes(routes!(controller::admin::remove_admin));

    // Discord routes
    let discord_routes = OpenApiRouter::new()
        .routes(routes!(controller::discord::get_all_discord_guilds))
        .routes(routes!(controller::discord::get_discord_guild_by_id))
        .routes(routes!(controller::discord::get_discord_guild_roles))
        .routes(routes!(controller::discord::get_discord_guild_channels));

    // Category routes
    let category_routes = OpenApiRouter::new()
        .routes(routes!(controller::category::get_fleet_categories))
        .routes(routes!(controller::category::create_fleet_category))
        .routes(routes!(controller::category::get_fleet_category_by_id))
        .routes(routes!(controller::category::update_fleet_category))
        .routes(routes!(controller::category::delete_fleet_category))
        .routes(routes!(
            controller::category::get_fleet_categories_by_ping_format
        ));

    // Ping format routes
    let ping_format_routes = OpenApiRouter::new()
        .routes(routes!(controller::ping_format::get_ping_formats))
        .routes(routes!(controller::ping_format::create_ping_format))
        .routes(routes!(controller::ping_format::update_ping_format))
        .routes(routes!(controller::ping_format::delete_ping_format));

    let ping_group_routes = OpenApiRouter::new()
        .routes(routes!(controller::ping_group::create_ping_group))
        .routes(routes!(controller::ping_group::get_paginated_ping_groups))
        .routes(routes!(controller::ping_group::update_ping_group))
        .routes(routes!(controller::ping_group::delete_ping_group));

    // Fleet routes
    let fleet_routes = OpenApiRouter::new()
        .routes(routes!(controller::fleet::get_guild_members))
        .routes(routes!(controller::fleet::get_category_details))
        .routes(routes!(controller::fleet::get_fleets))
        .routes(routes!(controller::fleet::create_fleet))
        .routes(routes!(controller::fleet::get_fleet))
        .routes(routes!(controller::fleet::update_fleet))
        .routes(routes!(controller::fleet::delete_fleet));

    // Combine all routes
    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(auth_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .merge(discord_routes)
        .merge(category_routes)
        .merge(ping_format_routes)
        .merge(ping_group_routes)
        .merge(fleet_routes)
        .split_for_parts();

    // Only serve Swagger UI in debug builds
    let api_router = if cfg!(debug_assertions) {
        api_router.merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", api))
    } else {
        api_router
    };

    // Configure CORS layer
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.clone())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true); // Required for session cookies

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(40)
        .burst_size(100)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    // Apply rate limiting and CORS layers to all routes
    Ok(api_router
        .layer(GovernorLayer::new(governor_conf))
        .layer(cors))
}
