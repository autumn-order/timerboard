use crate::server::{data::user::UserRepository, model::user::UpsertUserParam};
use sea_orm::DbErr;
use test_utils::builder::TestBuilder;

mod admin_exists;
mod find_by_discord_id;
mod get_all_admins;
mod get_all_paginated;
mod set_admin;
mod update_role_sync_timestamp;
mod update_role_sync_timestamps;
mod upsert;
