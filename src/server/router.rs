//! HTTP routing and OpenAPI documentation configuration.
//!
//! This module defines the application's HTTP routes and generates OpenAPI documentation
//! using utoipa. All API endpoints are registered here with their OpenAPI specifications,
//! and Swagger UI is configured to provide interactive API documentation at `/api/docs`.

use axum::Router;
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
    server::{controller, state::AppState},
};

/// Builds the application's HTTP router with all API endpoints and Swagger UI documentation.
///
/// Constructs an Axum router with all authentication, user management, admin, fleet, category,
/// and discord endpoints registered. Each endpoint is annotated with OpenAPI specifications via
/// utoipa, which are collected into a unified OpenAPI document. The router includes Swagger UI
/// at `/api/docs` for interactive API exploration and testing.
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
/// # Returns
/// An Axum `Router<AppState>` configured with all routes and middleware, ready to be
/// merged into the main application router.
///
/// # Example
/// ```ignore
/// let app_state = AppState { db, http_client, oauth_client, worker, admin_code_service };
/// let router = routes().with_state(app_state);
/// // Router is now ready to serve HTTP requests
/// ```
pub fn router() -> Router<AppState> {
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

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // Auth routes
        .routes(routes!(controller::auth::login))
        .routes(routes!(controller::auth::callback))
        .routes(routes!(controller::auth::logout))
        .routes(routes!(controller::auth::get_user))
        // User routes
        .routes(routes!(controller::user::get_user_guilds))
        .routes(routes!(controller::user::get_user_manageable_categories))
        // Admin routes
        .routes(routes!(controller::admin::add_bot))
        .routes(routes!(controller::admin::get_all_users))
        .routes(routes!(controller::admin::get_all_admins))
        .routes(routes!(controller::admin::add_admin))
        .routes(routes!(controller::admin::remove_admin))
        // Discord routes
        .routes(routes!(controller::discord::get_all_discord_guilds))
        .routes(routes!(controller::discord::get_discord_guild_by_id))
        .routes(routes!(controller::discord::get_discord_guild_roles))
        .routes(routes!(controller::discord::get_discord_guild_channels))
        // Category routes
        .routes(routes!(controller::category::get_fleet_categories))
        .routes(routes!(controller::category::create_fleet_category))
        .routes(routes!(controller::category::get_fleet_category_by_id))
        .routes(routes!(controller::category::update_fleet_category))
        .routes(routes!(controller::category::delete_fleet_category))
        .routes(routes!(
            controller::category::get_fleet_categories_by_ping_format
        ))
        // Ping format routes
        .routes(routes!(controller::ping_format::get_ping_formats))
        .routes(routes!(controller::ping_format::create_ping_format))
        .routes(routes!(controller::ping_format::update_ping_format))
        .routes(routes!(controller::ping_format::delete_ping_format))
        // Fleet routes
        .routes(routes!(controller::fleet::get_guild_members))
        .routes(routes!(controller::fleet::get_category_details))
        .routes(routes!(controller::fleet::get_fleets))
        .routes(routes!(controller::fleet::create_fleet))
        .routes(routes!(controller::fleet::get_fleet))
        .routes(routes!(controller::fleet::update_fleet))
        .routes(routes!(controller::fleet::delete_fleet))
        .split_for_parts();

    // Only serve Swagger UI in debug builds
    if cfg!(debug_assertions) {
        router.merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", api))
    } else {
        router
    }
}
