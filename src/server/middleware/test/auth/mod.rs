use crate::server::{
    data::category::FleetCategoryRepository,
    error::{auth::AuthError, AppError},
    middleware::{auth::AuthGuard, auth::Permission, session::AuthSession},
    model::category::{AccessRoleData, CreateFleetCategoryParams},
};
use test_utils::{builder::TestBuilder, factory};

mod require;
