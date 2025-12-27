use dioxus::prelude::*;

use crate::{client::model::error::ApiError, model::user::UserDto};

#[cfg(feature = "web")]
use crate::client::api::user::get_user;

#[derive(Clone, Copy)]
pub struct AuthContext {
    inner: Signal<AuthState>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            inner: Signal::new(AuthState::Initializing),
        }
    }

    pub fn read(&self) -> impl std::ops::Deref<Target = AuthState> + '_ {
        self.inner.read()
    }

    #[cfg(feature = "web")]
    pub fn fetch_user(&mut self) {
        let future = use_resource(get_user);
        if let Some(result) = &*future.read_unchecked() {
            let mut ctx = self.inner.write();
            *ctx = match result {
                Ok(Some(user)) => AuthState::Authenticated(user.clone()),
                Ok(None) => AuthState::NotLoggedIn,
                Err(e) => AuthState::Error(e.clone()),
            };
        }
    }
}

#[derive(Clone)]
pub enum AuthState {
    /// Initial state - haven't checked authentication yet
    Initializing,
    /// User is authenticated
    Authenticated(UserDto),
    /// No active session
    NotLoggedIn,
    /// Failed to check authentication
    Error(ApiError),
}

impl From<Option<UserDto>> for AuthState {
    fn from(opt: Option<UserDto>) -> Self {
        match opt {
            Some(user) => AuthState::Authenticated(user),
            None => AuthState::NotLoggedIn,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Permission {
    Admin,
}

impl AuthState {
    /// Check if the user is authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated(_))
    }

    /// Check if the user has admin permissions
    pub fn is_admin(&self) -> bool {
        match self {
            AuthState::Authenticated(user) => user.admin,
            _ => false,
        }
    }

    /// Check if the user has a specific permission
    ///
    /// Note: This only checks permissions for authenticated users.
    /// Being logged in is a prerequisite, not a permission.
    pub fn has_permission(&self, permission: &Permission) -> bool {
        match self {
            AuthState::Authenticated(user) => match permission {
                Permission::Admin => user.admin,
            },
            _ => false,
        }
    }

    /// Check if the user has all of the required permissions
    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|perm| self.has_permission(perm))
    }

    /// Get the authenticated user, if any
    pub fn user(&self) -> Option<&UserDto> {
        match self {
            AuthState::Authenticated(user) => Some(user),
            _ => None,
        }
    }

    pub fn user_id(&self) -> Option<u64> {
        self.user().map(|u| u.discord_id)
    }
}
