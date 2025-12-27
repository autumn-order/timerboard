use crate::model::user::UserDto;

#[derive(Clone)]
pub enum AuthState {
    Authenticated(UserDto),
    NotLoggedIn,
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
            AuthState::NotLoggedIn => false,
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
            AuthState::NotLoggedIn => false,
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
            AuthState::NotLoggedIn => None,
        }
    }

    pub fn user_id(&self) -> Option<u64> {
        match self {
            AuthState::Authenticated(user) => Some(user.discord_id),
            _ => None,
        }
    }
}
