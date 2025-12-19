use crate::server::data::discord::role::DiscordGuildRoleRepository;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter};
use serenity::all::RoleId;
use std::collections::HashMap;
use test_utils::{builder::TestBuilder, factory, serenity::create_test_role};

mod delete;
mod get_by_guild_id;
mod upsert;
mod upsert_many;
