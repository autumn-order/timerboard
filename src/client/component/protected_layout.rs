use dioxus::prelude::*;

use crate::client::{
    component::page::{ErrorPage, LoadingPage},
    model::auth::{AuthContext, AuthState, Permission},
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
    let auth_context = use_context::<AuthContext>();
    let nav = navigator();

    // Handle redirect for unauthenticated users
    {
        let auth_context = auth_context.clone();
        use_effect(move || {
            let state = auth_context.read();
            if matches!(&*state, AuthState::NotLoggedIn | AuthState::Error(_)) {
                nav.push(Route::Login {});
            }
        });
    }

    let state = auth_context.read();

    rsx! {
        match &*state {
            AuthState::Initializing => rsx! {
                LoadingPage {}
            },
            AuthState::NotLoggedIn | AuthState::Error(_) => rsx! {
                // Render nothing while redirecting
            },
            AuthState::Authenticated(_) => {
                if permissions.is_empty() || state.has_all_permissions(&permissions) {
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
