use dioxus::prelude::*;

use crate::{
    client::{
        component::page::{ErrorPage, LoadingPage},
        router::Route,
        store::user::UserState,
    },
    model::user::UserDto,
};

#[derive(PartialEq, Clone)]
pub enum Permission {
    LoggedIn,
    Admin,
}

#[component]
pub fn RequiresLoggedIn() -> Element {
    rsx! {
        ProtectedLayout { permissions: vec![Permission::LoggedIn] }
    }
}

#[component]
pub fn RequiresAdmin() -> Element {
    rsx! {
        ProtectedLayout { permissions: vec![Permission::Admin] }
    }
}

fn check_permissions(user: &Option<UserDto>, required_permissions: &[Permission]) -> bool {
    let user_data = match user {
        Some(u) => u,
        None => return false,
    };

    required_permissions.iter().all(|perm| match perm {
        Permission::LoggedIn => true,
        Permission::Admin => user_data.admin,
    })
}

#[component]
pub fn ProtectedLayout(permissions: Vec<Permission>) -> Element {
    let user_store = use_context::<Store<UserState>>();
    let nav = navigator();

    let user = user_store.read().user.clone();
    let fetch_completed = user_store.read().fetched;

    let user_logged_in = user.is_some();
    let has_required_permissions = check_permissions(&user, &permissions);

    // Redirect based on authentication and permissions
    use_effect(use_reactive!(|(user_logged_in, fetch_completed)| {
        if fetch_completed && !user_logged_in {
            nav.push(Route::Login {});
        }
    }));

    rsx! {
        // Show loading spinner while user is being fetched
        if !fetch_completed {
            LoadingPage {  }
        } else if user_logged_in && !has_required_permissions {
            ErrorPage { status: 403, message: "You don't have permission to view this page" }
        }
        // Render page if user is fetched, logged in, and has required permissions
        else if user_logged_in && has_required_permissions {
            Outlet::<Route> {}
        }
        // If fetched but doesn't have permissions, render nothing while redirecting
        // via the use_effect
    }
}
