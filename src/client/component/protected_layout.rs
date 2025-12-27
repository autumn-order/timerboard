use dioxus::prelude::*;

use crate::client::{
    component::page::{ErrorPage, LoadingPage},
    model::{
        auth::{AuthState, Permission},
        cache::{Cache, CacheState},
    },
    router::Route,
};

#[component]
pub fn RequiresLoggedIn() -> Element {
    rsx! {
        ProtectedLayout { permissions: vec![] }
    }
}

#[component]
pub fn RequiresAdmin() -> Element {
    rsx! {
        ProtectedLayout { permissions: vec![Permission::Admin] }
    }
}

#[component]
pub fn ProtectedLayout(permissions: Vec<Permission>) -> Element {
    let auth_cache = use_context::<Cache<AuthState>>();
    let nav = navigator();

    // Handle redirect for unauthenticated users
    {
        let auth_cache = auth_cache.clone();
        use_effect(move || {
            let cache = auth_cache.read();
            if matches!(
                &*cache,
                CacheState::Fetched(AuthState::NotLoggedIn) | CacheState::Error(_)
            ) {
                nav.push(Route::Login {});
            }
        });
    }

    let cache = auth_cache.read();

    rsx! {
        match &*cache {
            CacheState::NotFetched => rsx! {
                LoadingPage {}
            },
            CacheState::Fetched(AuthState::NotLoggedIn) | CacheState::Error(_) => rsx! {
                // Render nothing while redirecting
            },
            CacheState::Fetched(auth_state @ AuthState::Authenticated(_)) => {
                if permissions.is_empty() || auth_state.has_all_permissions(&permissions) {
                    rsx! {
                        Outlet::<Route> {}
                    }
                } else {
                    rsx! {
                        ErrorPage {
                            status: 403,
                            message: "You don't have permission to view this page"
                        }
                    }
                }
            }
        }
    }
}
