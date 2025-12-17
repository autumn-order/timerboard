# Dependencies

This application uses the Autumn tech stack which utilizes the Rust programming language for both:
- Frontend being Rust WASM via feature `web` with Dioxus frontend framework
- Server is native Rust via feature `server` with Axum API framework

Frontend (feature `web`):
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

 Uses: UserDto      →    Receives: UserDto   →    Uses: UserParam   →    Returns: UserParam
                        Converts to:               (CreateUserParam)       (converts entity)
                        CreateUserParam                                  
```

**Reverse flow** (returning the created user):

```
Database           →    Business Logic    →    API Endpoint       →    Frontend
                   
Returns:           →    UserParam         →    UserDto            →    Displays UserDto
UserParam               .into_dto()            (serialized JSON)
(converted from entity)
```

---

## The Five Layers (By Domain)

For each **domain** (e.g., `user`, `character`), we have these five pieces:

#### 1. **Data Repository** - `server/data/user.rs`
**Responsibility**: Database operations and entity-to-param conversion  
**Uses**: `entity::user::Model` internally, **returns**: `UserParam` (domain model)  
**Example**:

```rust
// Struct that provides required dependencies for all related repository methods
struct UserRepository<'a> {
    db: &'a DatabaseConnection
}

impl<'a> UserRepository<'a> {
    pub async fn create_user(&self, param: CreateUserParam) -> Result<UserParam> {
        // Insert into database using entity model
        let entity = // ... database insert operation
        
        // Convert entity to param at the infrastructure boundary
        Ok(UserParam::from_entity(entity))
    }
}
```

### 2. **Service** - `server/service/user.rs`
**Responsibility**: Business logic and orchestration  
**Uses**: `CreateUserParam`, `GetUserParam` (server-only param models)  
**Example**:

```rust
// Struct that provides required dependencies for all related service methods
struct UserService<'a> {
    db: &'a DatabaseConnection
}

impl<'a> UserService<'a> {
    pub async fn create_user(&self, param: CreateUserParam) -> Result<UserParam> {
        // Validate param
        // Call data repository (already returns param)
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
| **Param Model** | `server/model/user.rs` | ❌ | Data ↔ Service ↔ Controller | Domain models (business logic) |
| **DTO Model** | `model/user.rs` | ✅ | Controller ↔ Frontend | API data transfer (JSON) |

---

## Key Rules (What Goes Where)

✅ **DO:**
- Use **entity models** only inside `server/data/` functions (never return them)
- **Data layer returns param models** - convert entities to params at the infrastructure boundary
- Use **param models** between data/service/controller (server internal)
- Use **DTOs** only when crossing the API boundary (controller ↔ frontend)
- Implement `into_dto()` on param models in `server/model/{domain}.rs`
- Implement `from_entity()` on param models for data layer conversions

❌ **DON'T:**
- Don't return entity models from the data layer
- Don't let entity models leak into services or controllers
- Don't use DTOs inside services or data layer
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
   - Converts entity to UserParam immediately
   ↑ Returns UserParam

5. Service receives UserParam
   - Performs any additional business logic
   ↑ Returns UserParam

6. Controller receives UserParam
   - Calls param.into_dto()
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

/// Represents a user with full data (typically from database)
pub struct UserParam {
    pub id: i32,
    pub name: String,
    pub email: String,
}

/// Represents data needed to create a new user
pub struct CreateUserParam {
    pub name: String,
    pub email: String,
}

impl UserParam {
    /// Convert param to DTO for API responses
    pub fn into_dto(self) -> UserDto {
        UserDto {
            id: self.id,
            name: self.name,
            email: self.email,
        }
    }
    
    /// Convert entity model to param
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
/// - `LoggedIn`: Can only access this route if user is logged in
/// 
/// # Arguments
/// - `state`: Application state containing the database connection for character lookup
/// - `session`: User's session containing their user ID
/// 
/// # Returns
/// - `Ok(Some(UserDto))`: User's ID & name
/// - `Ok(None)`: User not in session or not in database
/// - `Err(DbErr(_))`: An error occurred retrieving user information from the database
/// - `Err(SessionErr(_))`: An error occurred getting user ID from session
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
    /// - `var`: Name of the environment variable with invalid value
    /// - `reason`: Explanation of why the value is invalid
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

# Dioxus 

You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

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
