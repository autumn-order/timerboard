# Dependencies

This application uses the Autumn tech stack which utilizes the Rust programming language for both

Frontend:
- `dioxus` - Fullstack frontend framework using Rust
- `DaisyUi` - CSS framework providing high-level CSS classes built atop tailwindcss
- `tailwindcss` - CSS library providing classes to quickly define styling
- `reqwasm` - For web-based API requests to backend (Allows us to use browser APIs such as cookies for sending requests with credentials to use session)
- `dioxus-free-icons` - For Font Awesome icons

Server (feature `server`):
- `tokio` - Async runtime
- `axum` - API framework
- `tower-sessions` - User session management
- `sea-orm` - Object relational management & migration management for database
- `tokio-cron-scheduler` - For periodic cron tasks
- `utoipa` - Swagger API doc generation for Axum-based APIs
- `reqwest` - For HTTP requests
- `dotenvy` - For loading `.env`

Database (feature `server`) - Check `Cargo.toml` to see which one this application uses
- `postgres` - for larger scale applications
- `sqlite` - for small scale applications
- Database migrations are found in the `migration/` folder
- Database entities are found in the `entity/` folder

Session (feature `server`) - Check `Cargo.toml` to see which one this application uses
- `redis` - Session store for larger applications
- `sqlite` - Session store for smaller applications

Situational (feature `server`)
- `eve_esi` - Rust API interface for EVE Online's ESI
- `serenity` - Rust API interface for Discord's API

# Domain-Driven Architecture

This application uses a **layered architecture** where data flows through distinct layers, each with its own responsibility and data model type. This separation prevents tight coupling and keeps concerns isolated.

---

## The Request Flow (Example: Creating a User)

```
Frontend                  Backend
--------                  -------
                         
Client Component    →    API Endpoint       →    Business Logic    →    Database
(Dioxus)                 (Controller)            (Service)              (Data Repository)

 Uses: UserDto      →    Receives: UserDto   →    Uses: User         →    Returns: User
                        Converts to:               (CreateUserParam)       (converts entity)
                        CreateUserParam                                  
```

**Reverse flow** (returning the created user):

```
Database           →    Business Logic    →    API Endpoint       →    Frontend
                   
Returns:           →    User              →    UserDto            →    Displays UserDto
User                    .into_dto()            (serialized JSON)
(converted from entity)
```

---

## The Five Layers (By Domain)

For each **domain** (e.g., `user`, `character`), we have these five pieces:

#### 1. **Data Repository** - `server/data/user.rs`
**Responsibility**: Database operations and entity-to-domain-model conversion  
**Uses**: `entity::user::Model` internally, **returns**: `User` (domain model)  
**Example**:

```rust
// Struct that provides required dependencies for all related repository methods
struct UserRepository<'a> {
    db: &'a DatabaseConnection
}

impl<'a> UserRepository<'a> {
    pub async fn create_user(&self, param: CreateUserParam) -> Result<User> {
        // Insert into database using entity model
        let entity = // ... database insert operation
        
        // Convert entity to domain model at the infrastructure boundary
        Ok(User::from_entity(entity))
    }
}
```

### 2. **Service** - `server/service/user.rs`
**Responsibility**: Business logic and orchestration  
**Uses**: `User` (domain model), `CreateUserParam`, `GetUserParam` (operation-specific params)  
**Example**:

```rust
// Struct that provides required dependencies for all related service methods
struct UserService<'a> {
    db: &'a DatabaseConnection
}

impl<'a> UserService<'a> {
    pub async fn create_user(&self, param: CreateUserParam) -> Result<User> {
        // Validate param
        // Call data repository (returns domain model)
        data::user::create_user(self.db, param).await
    }
}
```

### 3. **Controller** - `server/controller/user.rs`
**Responsibility**: Handle HTTP requests, access control, convert to DTOs  
**Uses**: Receives `CreateUserDto` from frontend, uses `CreateUserParam` internally, returns `UserDto`  
**Example**:

```rust
#[post("/users")]
pub async fn create_user(Json(dto): Json<CreateUserDto>) -> Result<Json<UserDto>> {
    // Check authentication
    let param: CreateUserParam = dto.into(); // Convert DTO to param
    let result = service::user::create_user(param).await?;
    Ok(Json(result.into_dto())) // Convert param to DTO
}
```

### 4. **Shared DTOs** - `model/user.rs`
**Responsibility**: Data transfer between frontend and backend via API  
**Uses**: `CreateUserDto`, `UserDto` (derives `Serialize`, `Deserialize`)  
**Example**:

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct UserDto {
    pub id: i32,
    pub name: String,
    pub email: String,
}
```

### 5. **Frontend API** - `client/api/user.rs`
**Responsibility**: Make API requests to backend  
**Uses**: `UserDto`, `CreateUserDto`  
**Example**:

```rust
pub async fn create_user(dto: CreateUserDto) -> Result<UserDto> {
    // POST request to /users with dto
}
```

---

## Data Models Explained

| Model Type | Location | Derives Serialize? | Used Where | Purpose |
|------------|----------|-------------------|------------|---------|
| **Entity Model** | `entity/user.rs` | ❌ | Data layer internally | Direct database representation (ORM) |
| **Domain Model** | `server/model/user.rs` | ❌ | Data ↔ Service ↔ Controller | Core business entities (e.g., `User`, `Character`) |
| **Param Model** | `server/model/user.rs` | ❌ | Service ↔ Controller | Operation-specific parameters (e.g., `CreateUserParam`, `UpdateUserParam`) |
| **DTO Model** | `model/user.rs` | ✅ | Controller ↔ Frontend | API data transfer (JSON) |

---

## Domain Models vs Parameter Models

It's important to distinguish between these two types in `server/model/`:

### Domain Models
**Purpose**: Represent complete business entities  
**Naming**: `{Domain}` (e.g., `User`, `Character`, `DiscordGuild`)  
**Usage**: Returned from repositories, passed between layers as complete entities

```rust
/// Represents a complete user entity
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}
```

### Parameter Models (`Param` Suffix)
**Purpose**: Contain data for specific operations  
**Naming**: `{Action}{Domain}Param` (e.g., `CreateUserParam`, `UpdateUserParam`, `GetUserParam`)  
**Usage**: Input parameters for operations that differ from the complete domain model

```rust
/// Parameters for creating a new user (no id yet)
pub struct CreateUserParam {
    pub name: String,
    pub email: String,
}

/// Parameters for querying a user
pub struct GetUserParam {
    pub id: i32,
}
```

**Rule of Thumb**: 
- If it represents a **complete entity**, don't use `Param` suffix → `User`, `Character`, `DiscordGuild`
- If it represents **operation-specific data**, use `Param` suffix → `CreateUserParam`, `UpdateTimerParam`

---

## Key Rules (What Goes Where)

✅ **DO:**
- Use **entity models** only inside `server/data/` functions (never return them)
- **Data layer returns domain models** - convert entities to domain models at the infrastructure boundary
- Use **domain models** as the primary type between data/service/controller layers
- Use **param models** for operation-specific input that differs from the domain model
- Use **DTOs** only when crossing the API boundary (controller ↔ frontend)
- Implement `into_dto()` on domain models in `server/model/{domain}.rs`
- Implement `from_entity()` on domain models for data layer conversions

❌ **DON'T:**
- Don't return entity models from the data layer
- Don't let entity models leak into services or controllers
- Don't use DTOs inside services or data layer
- Don't suffix domain models with `Param` (use `User`, not `UserParam`)
- Don't use `Param` suffix unless it's operation-specific input data
- Don't manually convert between models everywhere (use `into_dto()` and `From`/`Into` traits)

---

## Why This Architecture?

**Separation of Concerns**:
- **Database changes** only affect the data layer internals
- **Business logic changes** only affect the service layer
- **API contract changes** only affect DTOs and controllers

**Type Safety**:
- Param models enforce server-side validation rules
- DTOs enforce API contract
- Entity models match database schema exactly (hidden in data layer)

**No Tight Coupling**:
- Frontend never knows about database structure
- Services never know about HTTP details or ORM implementation
- Data layer returns domain models, not infrastructure types
- Can swap ORM implementations without changing services

---

## Complete Example Flow: "Get User by ID"

```
1. Frontend calls: client/api/user.rs::get_user(id: i32)
   ↓ Sends HTTP GET /users/123

2. Controller receives: server/controller/user.rs
   - Checks authentication
   - Extracts id from path
   - Creates GetUserParam { id: 123 }
   ↓ Calls service

3. Service processes: server/service/user.rs::get_user(param: GetUserParam)
   - Validates id
   - Calls data repository
   ↓ Calls data layer

4. Data layer queries: server/data/user.rs::get_user(param: GetUserParam)
   - Runs SQL: SELECT * FROM users WHERE id = 123
   - Gets entity::user::Model from database
   - Converts entity to User domain model immediately
   ↑ Returns User

5. Service receives User
   - Performs any additional business logic
   ↑ Returns User

6. Controller receives User
   - Calls user.into_dto()
   - Serializes to JSON
   ↑ Returns JSON UserDto

7. Frontend receives UserDto
   - Displays in component
```

---

## Conversion Helper Pattern

To avoid verbose conversion code, we implement these in `server/model/{domain}.rs`:

```rust
// In server/model/user.rs

/// Represents a complete user domain model
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

/// Parameters for creating a new user
pub struct CreateUserParam {
    pub name: String,
    pub email: String,
}

/// Parameters for querying a user
pub struct GetUserParam {
    pub id: i32,
}

impl User {
    /// Convert domain model to DTO for API responses
    pub fn into_dto(self) -> UserDto {
        UserDto {
            id: self.id,
            name: self.name,
            email: self.email,
        }
    }
    
    /// Convert entity model to domain model at repository boundary
    pub fn from_entity(entity: entity::user::Model) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            email: entity.email,
        }
    }
}

impl From<CreateUserDto> for CreateUserParam {
    /// Convert DTO to param when receiving API requests
    fn from(dto: CreateUserDto) -> Self {
        Self {
            name: dto.name,
            email: dto.email,
        }
    }
}
```

---

# File Structure

## Root Overview

```
application/
├── src/
│   ├── client/          # Dioxus frontend UI
│   ├── model/           # Shared DTOs between backend & frontend
│   ├── server/          # Axum API backend
│   └── main.rs          # Primary application entry point
├── entity/              # Database entity models (generated by sea-orm)
├── migration/           # Database migration files
├── Cargo.toml           # Dependency management
├── .env.example         # Environment variables template
├── docker-compose.yml   # Production docker configuration
├── docker-compose.dev.yml # Development dependencies (Postgres, Redis)
├── Dockerfile           # Configuration for building docker image
├── package.json         # CSS dependencies installed via `bun`
└── tailwind.css         # CSS configuration such as DaisyUi themes
```

---

## Client Overview

```
client/
├── api/                 # API request handlers (reqwasm-based, WASM only)
│   └── {domain}.rs      # e.g., user.rs, character.rs, timer.rs
├── component/           # Reusable UI components shared across routes
│   └── *.rs             # Individual component files
├── model/               # Frontend-only data structures
│   └── *.rs             # Client-specific models
├── route/               # Application pages (folder-based routing)
│   ├── {page}/          # Each route is a folder
│   │   ├── component/   # Components specific to this route only
│   │   └── mod.rs       # The page component itself
│   └── mod.rs           # Route exports
├── store/               # Global application state (accessible frontend-wide)
│   └── *.rs             # Context/signal stores
├── app.rs               # Frontend root with context providers (theme, user, etc.)
├── constant.rs          # Frontend constants
├── router.rs            # Router enum defining all frontend routes
└── mod.rs               # Client module exports
```

---

## Model Overview

```
model/
├── api.rs               # Common API DTOs (ErrorDto, SuccessDto, etc.)
├── user.rs              # User domain DTOs (UserDto, CreateUserDto, etc.)
└── {domain}.rs          # One file per domain with relevant DTOs
```

---

## Server Overview

```
server/
├── bot/                 # Discord bot integration (if applicable)
│   ├── handler/         # Discord event handlers
│   │   ├── ready.rs
│   │   ├── message.rs
│   │   └── mod.rs
│   ├── start.rs         # Bot initialization and startup
│   └── mod.rs
├── controller/          # API request handlers & access control
│   ├── {domain}.rs      # e.g., user.rs, character.rs, timer.rs
│   └── mod.rs           # Controller router setup
├── data/                # Database repositories (CRUD operations)
│   ├── {domain}.rs      # Uses entity::* models
│   └── mod.rs
├── error/               # Error handling & custom error types
│   ├── auth.rs          # Authentication-specific errors
│   ├── config.rs        # Configuration errors
│   ├── mod.rs           # AppError enum (primary error type)
│   └── {domain}.rs      # Domain-specific errors
├── middleware/          # Request/response middleware
│   ├── session/         # Session management structs
│   │   └── *.rs
│   ├── auth_guard.rs    # Authentication middleware
│   └── mod.rs
├── model/               # Server-only parameter models (no Serialize)
│   ├── {domain}.rs      # e.g., CreateUserParam, GetUserParam
│   └── mod.rs           # Includes `into_dto()` implementations
├── scheduler/           # Cron jobs & periodic tasks
│   ├── {task}.rs        # Individual scheduled tasks
│   └── mod.rs           # tokio-cron-scheduler configuration
├── service/             # Business logic layer
│   ├── {domain}.rs      # Logic between data & controller layers
│   └── mod.rs
├── config.rs            # AppConfig struct with `from_env()` method
├── router.rs            # Axum router configuration
├── startup.rs           # Server initialization (DB, session, etc.)
├── state.rs             # AppState (DB pools, session, shared resources)
└── mod.rs               # Server module exports
```

---

# Documentation Standards

## Methods

- We list the arguments and what the argument is for, let the code speak for itself, our docs explain the why and the behavior
- For arguments we describe briefly what that argument is used for
- For return types, we should list out each possible variant & error as well as why it returned that value

### Service/Data Methods

- Short description of what the method does
- 2-3 sentences on behavior
- List of arguments and what the arguments are/do
- List of all response variants included possible errors

Example:

```rust
/// Generates an OAuth2 login URL for EVE Online SSO.
///
/// Creates a login URL with the requested scopes and a CSRF state token for security.
/// The user should be redirected to this URL to begin the authentication flow with EVE Online.
///
/// # Arguments
/// - `scopes` - List of OAuth2 scopes to request from the user
///
/// # Returns
/// - `Ok(AuthenticationData)` - Login URL and CSRF state token for callback validation
/// - `Err(Error::EsiError)` - ESI client OAuth2 not configured properly
pub fn generate_login_url(&self, scopes: Vec<String>) -> Result<AuthenticationData, Error> {
    let login = self.esi_client.oauth2().login_url(scopes)?;

    Ok(login)
}
```

### Controller Methods

- We use the utoipa::path proc macro for generating SwaggerUi API documentation
- We do `///` method documentation to describe behavior which will also be shown in the SwaggerUi
- For access control, these are generally permission enum variants such as `Permission::LoggedIn`

Example:

```rust
static USER_TAG: &str = "user";

/// Retrieves information for the currently authenticated user
///
/// Fetches the user ID from the session & checks to see if they are logged in. Then queries
/// the database to retrieve information on the user, returning their user ID & name.
/// 
/// # Access Control
/// - `LoggedIn` - Can only access this route if user is logged in
/// 
/// # Arguments
/// - `state` - Application state containing the database connection for character lookup
/// - `session` - User's session containing their user ID
/// 
/// # Returns
/// - `Ok(Some(UserDto))` - User's ID & name
/// - `Ok(None)` - User not in session or not in database
/// - `Err(DbErr(_))` - An error occurred retrieving user information from the database
/// - `Err(SessionErr(_))` - An error occurred getting user ID from session
#[utoipa::path(
    get,
    path = "/api/user",
    tag = USER_TAG,
    responses(
        (status = 200, description = "Success when retrieving user", body = UserDto),
        (status = 404, description = "User not found", body = ErrorDto),
        (status = 500, description = "Internal server error", body = ErrorDto)
    ),
)]
pub async fn get_user(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    // AuthGuard would retrieve user info when doing permission check so we can just
    // return the user from auth guard to avoid redundant DB calls
    let user = AuthGuard::new(&state.db, &session)
        .require(&[Permission::LoggedIn])
        .await?;

    Ok((StatusCode::OK, Json(user.into_dto())).into_response())
}
```


### Test Methods

- Short description of expected result
- 2-3 sentences on what is being verified and the current state
- Expected outcome

Example:

```rust
/// Tests successful redirect to EVE Online login page.
///
/// Verifies that the login endpoint returns a 307 temporary redirect response
/// that directs the user to the EVE Online SSO login page for authentication.
///
/// Expected: Ok with 307 TEMPORARY_REDIRECT response
#[tokio::test]
async fn redirects_to_eve_login() -> Result<(), TestError> {
    let test = TestBuilder::new().build().await?;

    let params = LoginParams { change_main: None };
    let result = login(State(test.into_app_state()), test.session, Query(params)).await;

    assert!(result.is_ok());
    let resp = result.unwrap().into_response();
    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);

    Ok(())
}
```

---

## Modules

- Short description of the module
- 1-3 paragraphs depending on complexity that describs what the module does

Example:

```rust
//! OAuth2 callback service for EVE Online SSO authentication.
//!
//! This module provides the `CallbackService` for handling OAuth2 callbacks from EVE SSO.
//! It orchestrates token validation, character ownership management, user creation/updates,
//! and main character assignment with comprehensive retry logic and caching.
```

---

## Enums

- Short description of what the enum represents
- 2-3 sentences of its purpose and behavior
- Each variant should have a short description, what it is for, and, if applicable, what its fields are for.

Example:

```rust
/// Configuration error type for environment variable validation failures.
///
/// These errors occur during application startup when the configuration system detects
/// missing or invalid environment variables. Configuration errors are always treated as
/// fatal and result in 500 Internal Server Error responses if encountered during request
/// handling, though typically they prevent the application from starting at all.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Required environment variable is not set.
    ///
    /// The application requires this environment variable to be defined. Check the
    /// documentation or `.env.example` file for required configuration variables.
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    /// Environment variable value is invalid or malformed.
    ///
    /// The environment variable is set but contains a value that cannot be parsed or
    /// is not within acceptable bounds. The `reason` field provides details about why
    /// the value was rejected.
    ///
    /// # Fields
    /// - `var` - Name of the environment variable with invalid value.
    /// - `reason` - Explanation of why the value is invalid.
    #[error("Invalid value for environment variable {var}: {reason}")]
    InvalidEnvValue {
        /// Name of the environment variable with invalid value.
        var: String,
        /// Explanation of why the value is invalid.
        reason: String,
    },
}
```

---

## Structs

- Short description
- 2-3 sentences on behavior, purpose, and/or what the struct represents
- Each field should have its own `///` documentation describing what it represents

Example:

```rust
/// Worker job with scheduled execution timestamp.
///
/// Wraps a `WorkerJob` with the timestamp it was scheduled for execution. This allows
/// the worker handler to distinguish between jobs scheduled before ESI downtime (which
/// should be rescheduled) versus jobs scheduled during downtime (scheduler bug).
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledWorkerJob {
    /// The worker job to execute.
    pub job: WorkerJob,
    /// The UTC timestamp when this job was scheduled for execution.
    pub scheduled_at: DateTime<Utc>,
}
```

---

# Testing

## How we are testing

**Unit Testing**
- Unit testing is anything that involves a single method under test with minimal dependencies such as a database repository method or a helper method
- `DatabaseConnection` - we use in-memory `sqlite` with only the required tables (foreign key-related) for test purposes, this allows us to unit test database repository methods instead of integration testing them
- `Session` - we use an in-memory `sqite` driver for `tower-sessions`, allowing us to also unit test anything session-related

**Integration Testing**
- Integration testing is more complex logic that has dependencies on multiple underlying methods or external providers like redis or mockito for mock HTTP servers. We would integration any services, controllers, worker handlers, schedulers, etc
- `Mockito` - allows us to create mock HTTP servers, due to involvement of mock HTTP servers we would make any test that relies on this an integration test
- `Redis` - we would need to use a development redis instance making anything using redis an integration test

## What we are testing

Execution paths (all functions):
- Ok return types (Some, None, Vec empty, Vec single, Vec multiple, etc)
- Error variant return types (DbErr, SessionErr, any `?` error propagation so we know how it is triggered)
- Conditional branches (match, if, if else, if let Some())
- We want to ensure all execution paths return the result as expected

Database behavior (repository/service):
- What happens if we violate a foreign key constraint, what error would we expect (Sqlite does enforce foreign keys)
- What happens if we violate a unique index, are we properly enforcing this? (Sqlite doesn't generally enforce this so we should add additional code to the method to handle it ourselves)
- We want to ensure that the method under test creates the exact results we expect

Permissions Access (controllers):
- We should get a forbidden response if we try to request this resource with improper permisions
- We should get a success response regardless of permissions if we are admin

We use code coverage tools such as `cargo llvm-cov` to track what % of coverage we have on execution paths within our codebase, we aim to get around 95% coverage of execution paths in total.

---

## Testing Structure

### Test methods

Test method structure is:
- Setup: create the starting state before we execute the method under test
- Execution: run the method under test
- Assertions: run assertions such as we got the expected values, the values were properly inserted into database, and expected requests were made to the http server

Integration test example:

```rust
/// Tests updating a new character without faction.
///
/// Verifies that the character service successfully fetches character data from ESI
/// and creates a new character record in the database when the character has no
/// faction affiliation.
///
/// Expected: Ok with character record created
#[tokio::test]
async fn updates_new_character_without_faction() -> Result<(), TestError> {
    // Test setup using a builder pattern
    let character_id = 95_000_001;
    let corporation_id = 98_000_001;

    let test = TestBuilder::new()
        // Migrate required tables for the test on an in-memory sqlite instance
        // - We only migrate the tables we actually need for the current test
        .with_table(entity::prelude::EveFaction)
        .with_table(entity::prelude::EveAlliance)
        .with_table(entity::prelude::EveCorporation)
        .with_table(entity::prelude::EveCharacter)
        // Create any required mockito mock HTTP endpoints that will be requested
        // - Each endpoint will expect 1 request each
        .with_corporation_endpoint(corporation_id, factory::mock_corporation(None, None), 1)
        .with_character_endpoint(
            character_id,
            factory::mock_character(corporation_id, None, None),
            1,
        )
        .build()
        .await?;

    // Execute the method under test
    let character_service = CharacterService::new(&test.db, &test.esi_client);
    let result = character_service.update(character_id).await;

    // Go through all assertions to ensure we have the result & state we expected
    assert!(result.is_ok());
    let character = result.unwrap();
    assert_eq!(character.character_id, character_id);
    assert!(character.faction_id.is_none());

    // Verify character exists in database
    let db_character = entity::prelude::EveCharacter::find()
        .one(&test.db)
        .await?
        .unwrap();
    assert_eq!(db_character.character_id, character_id);
    assert!(db_character.faction_id.is_none());

    // Verify corporation was created in database
    let corporation = entity::prelude::EveCorporation::find()
        .one(&test.db)
        .await?;
    assert!(corporation.is_some());

    // Assert expected HTTP requests were made to the mock server
    test.assert_mocks();

    Ok(())
}
```

### Inline Tests (Minor Unit Tests)

We use inline tests for unit testing small helper methods, keeping the test module in the same file of the methods under test. For instances of larger objects such as repositories, we would prefer folder and file tests instead so as to not make the file
of the methods being tested too lengthy.

- Use `mod test` as the root
- For each method, add a new mod, e.g. `mod user`
- Favor usage of `use super::*;` to reduce verbosity of imports
- Use the `Result<(), Error>` return pattern so we can propagate errors with `?` to be concise

Example:

```rust
#[cfg(test)]
mod test {
    use super::*;

    mod create_user {
        use super::*;
        
        /// Tests creating a new user.
        ///
        /// Verifies that the user repository successfully creates a new user record
        /// with the specified main character ID.
        ///
        /// Expected: Ok
        #[tokio::test]
        async fn creates_user() -> Result<(), AppError> {
            // Test logic goes here
            todo!()
    
            Ok(())
        }

    }
}
```

### Folder & File Tests (Unit & Integration Tests)

We use folder & file-based test structure for integration tests and for unit tests if the file we are doing inline unit tests in begins to near reaching over `500 lines`.

- We do a test folder at the root of where we are testing files (`test/` folder in same folder as `user.rs`)
- Then, the test folder contains a folder that represents each file we are testing (`test/user/` for `user.rs`)
- The folder for the `user.rs` tests then contains a file for each method under test (`test/user/get_user.rs` for get_user method in `user.rs`)
- Each test module will import `super::*;` to reduce repetition in imports

Example folder structure:

```
data/
├── test/
│   ├── mod.rs              # Imports `super::*;`
│   └── user/               # Contains tests for all `user.rs` methods
│       ├── mod.rs          # Imports `super::*;`
│       └── get_user.rs     # Has tests solely for `get_user` method
└── user.rs                 # Contains `get_user` method
```

Example method test file:

```rust
use super::*;

/// Tests error handling for nonexistent main character.
///
/// Verifies that the user repository returns an error when attempting to create
/// a user with a main character ID that does not exist in the database.
///
/// Expected: Err
#[tokio::test]
async fn fails_for_nonexistent_main_character() -> Result<(), AppError> {
    // Test logic goes here
    todo!()

    Ok(())
}

// Additional test methods...
```

---

## Test Factories

To reduce boilerplate and improve test maintainability, we use factory methods from the `test-utils` crate for creating test entities with sensible defaults.

### Factory Usage

**Basic entity creation:**

```rust
use test_utils::factory;

#[tokio::test]
async fn test_example() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_fleet_message_tables()
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    // Create individual entities with defaults
    let user = factory::create_user(db).await?;
    let guild = factory::create_guild(db).await?;
    
    Ok(())
}
```

**Creating entity hierarchies:**

```rust
// Create a complete fleet hierarchy with all dependencies
let (user, guild, ping_format, category, fleet) =
    factory::helpers::create_fleet_with_dependencies(db).await?;
```

**Customizing entities:**

```rust
use test_utils::factory::user::UserFactory;

// Create admin user with custom values
let admin = UserFactory::new(db)
    .discord_id("123456789")
    .name("AdminUser")
    .admin(true)
    .build()
    .await?;
```

**Reusing dependencies:**

```rust
// Create first fleet with all dependencies
let (user, _guild, _ping_format, category, fleet1) =
    factory::helpers::create_fleet_with_dependencies(db).await?;

// Create second fleet using same category and commander
let fleet2 = factory::create_fleet(db, category.id, &user.discord_id).await?;
```

### Table Setup Convenience Methods

The `TestBuilder` provides convenience methods for common table groupings:

```rust
// For fleet-related tests (User, Guild, PingFormat, Category, Fleet)
let test = TestBuilder::new()
    .with_fleet_tables()
    .build()
    .await
    .unwrap();

// For fleet message tests (includes FleetMessage table)
let test = TestBuilder::new()
    .with_fleet_message_tables()
    .build()
    .await
    .unwrap();
```

### Available Factories

- `factory::user` - Create user entities
- `factory::discord_guild` - Create Discord guild entities  
- `factory::ping_format` - Create ping format entities
- `factory::fleet_category` - Create fleet category entities
- `factory::fleet` - Create fleet entities
- `factory::helpers` - Composite creation methods for entity hierarchies

### Benefits

- **Reduced boilerplate**: Tests focus on behavior, not setup
- **Maintainability**: Entity structure changes only require factory updates
- **Consistency**: All tests use the same entity creation patterns
- **Readability**: Clear intent with descriptive factory methods

---

# Logging

- We import `dioxus_logger::tracing` for logging, then using `tracing::info!` macros to log information
- It is important we avoid logging noise, only utilizing 

- `error!` logging belongs either top level or if we get an error that causes us to have to skip something, such as skipping update for a resource due to a not found in a batch update method, but doesn't cause an error to propagate upwards. 
- When errors are fully propagated upwards, we would log them in the top level such as the worker job handler or `into_response` method we impl on the `AppError` enum for errors within controllers. That way we only error log in a single place for the propagated error.
- `warn!` logging is used when we have unexpected, but non-fatal state which is recoverable but may result in degraded functionality. We should `warn!` log for it to indicate issues may arise but there aren't any major, application-breaking problems. We do so for example where we can safely set `None` for a value as a temporary solution but may cause data not to display as expected.
- `info!` we use for significant events that effect application-wide state, a routine API request we may `debug!` log but we wouldn't `info!` log it. Now, if we for example add a new admin, we'd then `info!` log it. Or if we refreshed a cache the entire application relies on that we update hourly, we would `info!` log that as well.
- `debug!` is used to help us examine the behavior of the application such as we run a periodic cron task or make an API fetch: "Ran periodic update task for user ID {} at {}", "skipping update task scheduled for {}, already up to date", "successfully fetched x from {endpoint} after x seconds".
- `trace!` would be used when we enter/exit a function such as "getting user from database" at the start of a repo method & "successfully found user in database" at the end of a repo method before returning.

---

# Naming Conventions

Consistent naming conventions across the codebase ensure code is predictable, maintainable, and easy to navigate. These conventions are organized by the type of item being named.

## Data Transfer Objects (DTOs)

DTOs are used for data transfer between frontend and backend via API endpoints.

**Format**: `{Action}{Domain}Dto`

**Location**: `model/{domain}.rs`

**Examples**:
- `UserDto` - Full user data for display
- `CreateUserDto` - Data required to create a new user
- `UpdateTimerDto` - Data required to update a timer
- `GetCharacterDto` - Character data returned from API

**Rules**:
- Always use `Dto` suffix
- Use descriptive action prefixes: `Create`, `Update`, `Get`, `Delete`
- Must derive `Serialize` and `Deserialize`

## Domain Models (Server-Only)

Domain models represent complete business entities used throughout the server layers.

**Format**: `{Domain}` (no suffix - e.g., `User`, `Character`, `DiscordGuild`)

**Location**: `server/model/{domain}.rs`

**Examples**:
- `User` - Complete user domain model
- `Character` - Complete character domain model
- `DiscordGuild` - Complete Discord guild domain model
- `Timer` - Complete timer domain model

**Rules**:
- NO `Param` suffix for domain models (use `User`, not `UserParam`)
- Must NOT derive `Serialize` or `Deserialize` (server-only)
- Implement `into_dto()` method for conversion to DTOs
- Implement `from_entity()` for conversion from entity models
- Represent the complete state of a business entity

## Parameter Models (Server-Only)

Param models are operation-specific input types that differ from the complete domain model.

**Format**: `{Action}{Domain}Param` (singular - use `Param`, not `Params`)

**Location**: `server/model/{domain}.rs`

**Examples**:
- `CreateUserParam` - Data required to create a user (no id yet)
- `UpdateUserParam` - Data required to update a user
- `GetUserParam` - Parameters for fetching a user (just id)
- `UpdateTimerParam` - Parameters for updating a timer

**Rules**:
- ONLY use `Param` suffix for operation-specific input types
- Use action prefixes: `Create`, `Update`, `Get`, `Delete`, `Upsert`
- Must NOT derive `Serialize` or `Deserialize` (server-only)
- Should contain only the fields needed for that specific operation
- If the operation uses the full domain model, don't create a param - use the domain model directly

## Repository Structs

Repository structs provide database operations for a specific domain.

**Format**: `{Domain}Repository`

**Location**: `server/data/{domain}.rs`

**Examples**:
- `UserRepository` - Database operations for users
- `CharacterRepository` - Database operations for characters
- `TimerRepository` - Database operations for timers

**Rules**:
- Always use `Repository` suffix
- Contain a lifetime parameter for database connection: `Repository<'a>`
- Hold a `db: &'a DatabaseConnection` field
- Methods accept operation-specific params (e.g., `CreateUserParam`) as input
- Methods return domain models (e.g., `User`, `Character`) as output
- Convert entity models to domain models at the repository boundary using `from_entity()`

## Service Structs

Service structs contain business logic between data and controller layers.

**Format**: `{Domain}Service`

**Location**: `server/service/{domain}.rs`

**Examples**:
- `UserService` - Business logic for user operations
- `CharacterService` - Business logic for character operations
- `AuthService` - Business logic for authentication

**Rules**:
- Always use `Service` suffix
- Contain a lifetime parameter: `Service<'a>`
- Hold a `db: &'a DatabaseConnection` field or any other fields required to be shared across service methods
- Methods work primarily with domain models (e.g., `User`, `Character`)
- Methods accept operation-specific params (e.g., `CreateUserParam`) for operations
- Methods return domain models (e.g., `User`, `Character`) to controllers

## Controller Functions

Controller functions handle HTTP requests and responses.

**Format**: `{action}_{domain}`

**Location**: `server/controller/{domain}.rs`

**Examples**:
- `create_user` - POST endpoint to create user
- `get_user` - GET endpoint to retrieve user
- `delete_timer` - DELETE endpoint to remove timer
- `update_character` - PUT/PATCH endpoint to update character

**Rules**:
- Prefix with HTTP action verb: `get_`, `create_`, `update_`, `delete_`, `list_`
- Must be `pub async fn`
- Use `#[utoipa::path]` macro for Swagger documentation
- Return `Result<impl IntoResponse, AppError>`

## File Naming

All files follow consistent naming patterns based on their domain.

**Format**: `{domain}.rs` (lowercase, snake_case)

**Examples**:
- `user.rs` - User domain
- `character.rs` - Character domain
- `eve_corporation.rs` - EVE Corporation domain (multi-word)
- `auth_guard.rs` - Authentication guard (multi-word)

**Rules**:
- One domain per file
- Use `snake_case` for multi-word domains
- Keep names concise but descriptive

## Constants

Constants use screaming snake case and are typically defined at the module level.

**Format**: `SCREAMING_SNAKE_CASE`

**Examples**:
- `USER_TAG` - Tag for Swagger UI grouping
- `MAX_RETRIES` - Maximum retry attempts
- `DEFAULT_TIMEOUT` - Default timeout duration
- `API_BASE_URL` - Base URL for API requests

**Rules**:
- Always use `SCREAMING_SNAKE_CASE`
- Declare with `static` or `const`
- Use descriptive names that indicate purpose

## Enums

Enums represent a type with multiple variants.

**Format**: `{Description}` (PascalCase)

**Examples**:
- `ConfigError` - Configuration error types
- `Permission` - Permission levels
- `WorkerJob` - Types of worker jobs
- `Route` - Application routes (Dioxus routing)

**Rules**:
- Use `PascalCase` for enum names
- Variants also use `PascalCase`: `LoggedIn`, `Admin`, `MissingEnvVar`
- Error enums should have `Error` suffix
- Derive appropriate traits: `Debug`, `Clone`, `PartialEq` as needed

## Error Types

Error types are enums that represent different error conditions.

**Format**: `{Domain}Error` or `AppError`

**Location**: `server/error/{domain}.rs` or `server/error/mod.rs`

**Examples**:
- `AppError` - Top-level application error enum
- `ConfigError` - Configuration-specific errors
- `AuthError` - Authentication-specific errors
- `DatabaseError` - Database operation errors

**Rules**:
- Enum names always use `Error` suffix (e.g., `AppError`, `ConfigError`)
- Variants do NOT use `Error` suffix to avoid clippy `enum_variant_names` warning
- Must derive `Error` and `Debug` from `thiserror`
- Use `#[error("...")]` attributes for error messages
- Variants use `PascalCase`: `NotFound`, `InvalidInput`, `Unauthorized`, `MissingEnvVar`

**Example**:
```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,  // ✅ Good - no Error suffix
    
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),  // ✅ Good
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),  // ✅ Good - wraps another error type
}

// ❌ Bad - clippy will warn about redundant suffix
pub enum AppError {
    NotFoundError,  // Redundant - already in AppError enum
    DatabaseError,  // Redundant
}
```

## Test Modules

Test modules follow a nested structure based on the methods being tested.

**Format**: 
- Root module: `mod test`
- Method modules: `mod {method_name}`
- Test functions: `{concise_description_of_test}` (snake_case)

**Location**: Inline in same file or `test/{file}/{method}.rs`

**Examples**:
```rust
#[cfg(test)]
mod test {
    use super::*;
    
    mod create_user {
        use super::*;
        
        #[tokio::test]
        async fn creates_user_successfully() -> Result<(), AppError> {
            // Test implementation
        }
        
        #[tokio::test]
        async fn fails_for_duplicate_email() -> Result<(), AppError> {
            // Test implementation
        }
    }
    
    mod get_user {
        use super::*;
        
        #[tokio::test]
        async fn returns_existing_user() -> Result<(), AppError> {
            // Test implementation
        }
    }
}
```

**Rules**:
- Root test module always named `mod test`
- Each method gets its own nested module
- Test function names describe what they test in `snake_case`
- Use descriptive names: `creates_x`, `fails_for_y`, `returns_z`, `updates_x_when_y`

## Frontend Components

Dioxus components follow specific naming conventions.

**Format**: `{ComponentName}` (PascalCase) for component functions

**Location**: `client/component/{name}.rs` or `client/route/{page}/component/{name}.rs`

**Examples**:
- `NavBar` - Navigation bar component
- `UserCard` - Card displaying user information
- `TimerList` - List of timers component
- `LoginForm` - Login form component

**Rules**:
- Component functions use `PascalCase` or contain underscore
- Must have `#[component]` macro
- File names use `snake_case`: `nav_bar.rs`, `user_card.rs`
- Props use `snake_case`: `user_id`, `is_active`, `on_click`

## Variables and Fields

General variable naming follows Rust conventions.

**Format**: `snake_case`

**Examples**:
- `user_id` - User identifier
- `character_name` - Character name
- `main_character_id` - Main character identifier
- `is_admin` - Boolean flag
- `created_at` - Timestamp field

**Rules**:
- Always use `snake_case`
- Boolean fields prefixed with `is_`, `has_`, `can_`, `should_`
- Avoid abbreviations unless widely understood (`id`, `url`, `api`)
- Be descriptive but concise

---

# Dioxus

You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.2" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

# Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

* Props must be owned values, not references. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
* Props must implement `PartialEq` and `Clone`.
* To make props reactive and copy, you can wrap the type in `ReadOnlySignal`. Any reactive state like memos and resources that read `ReadOnlySignal` props will automatically re-run when the prop changes.

# State

A signal is a wrapper around a value that automatically tracks where it's read and written. Changing a signal's value causes code that relies on the signal to rerun.

## Local State

The `use_signal` hook creates state that is local to a single component. You can call the signal like a function (e.g. `my_signal()`) to clone the value, or use `.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its dependencies change. Memos are useful for expensive calculations that you don't want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

## Context API

The Context API allows you to share state down the component tree. A parent provides the state using `use_context_provider`, and any child can access it with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

# Async

For state that depends on an asynchronous operation (like a network request), Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of the async task and provides the result to your component.

* The `use_resource` hook takes an `async` closure. It re-runs this closure whenever any signals it depends on (reads) are updated
* The `Resource` object returned can be in several states when read:
1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

# Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`. Each variant represents a route and is annotated with `#[route("/path")]`. Dynamic Segments can capture parts of the URL path as parameters by using `:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and place an `Outlet<Route> {}` inside your layout component. The child routes will be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

# Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features (`server` and a client feature like `web`) to split the code into a server and client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

## Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only run on the server. On the server, this macro generates an API endpoint. On the client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

## Hydration

Hydration is the process of making a server-rendered HTML page interactive on the client. The server sends the initial HTML, and then the client-side runs, attaches event listeners, and takes control of future rendering.

### Errors
The initial UI rendered by the component on the client must be identical to the UI rendered on the server.

* Use the `use_server_future` hook instead of `use_resource`. It runs the future on the server, serializes the result, and sends it to the client, ensuring the client has the data immediately for its first render.
* Any code that relies on browser-specific APIs (like accessing `localStorage`) must be run *after* hydration. Place this code inside a `use_effect` hook.

## WASM Usage

The `client` folder is available on feature `server` as well and should be as such as it is required by `dx serve`. It should be noted though, any usage of WASM libraries such as `web-sys`, `gloo-timers`, or `reqwasm` must be within a closure, method, or module behind feature `web`. WASM functions cannot be used within feature `server`.

Example (`client/mod.rs`):

```rust
pub mod app;
pub mod component;
pub mod constant;
pub mod model;
pub mod route;
pub mod router;
pub mod store;

#[cfg(feature = "web")]
pub mod api;

pub use app::App;
```

Notice how all modules are available to both `server` & `web` except for `api` which contains only `reqwasm`-based methods to fetch data from the backend.

We would then use these methods like so:

```rust
// Imports shared between `web` & `server`
use dioxus::prelude::*;
use dioxus_logger::tracing;

// We import the API method only for feature `web` since it is WASM-only
#[cfg(feature = "web")]
use crate::client::api::ping_format::delete_ping_format;

#[component]
pub fn PingFormatsTable() -> Element {
    // We gate the usage of the reqwasm API method in a closure behind feature `web`
    #[cfg(feature = "web")]
    let delete_future = use_resource(move || async move {
        if is_deleting() {
            if let Some((id, _, _)) = format_to_delete() {
                Some(delete_ping_format(guild_id, id).await)
            } else {
                None
            }
        } else {
            None
        }
    });
    
    todo!()
}
```
