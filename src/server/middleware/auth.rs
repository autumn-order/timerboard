use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::server::{
    controller::auth::SESSION_AUTH_USER_ID,
    data::user::UserRepository,
    error::{auth::AuthError, AppError},
};

pub enum Permission {
    Admin,
    CategoryView(u64, i32),   // guild_id, category_id
    CategoryCreate(u64, i32), // guild_id, category_id
}

pub struct AuthGuard<'a> {
    db: &'a DatabaseConnection,
    session: &'a Session,
}

impl<'a> AuthGuard<'a> {
    pub fn new(db: &'a DatabaseConnection, session: &'a Session) -> Self {
        Self { db, session }
    }

    pub async fn require(
        &self,
        permissions: &[Permission],
    ) -> Result<entity::user::Model, AppError> {
        let user_repo = UserRepository::new(self.db);

        let Some(user_id_str) = self.session.get::<String>(SESSION_AUTH_USER_ID).await? else {
            return Err(AuthError::UserNotInSession.into());
        };

        let user_id = user_id_str.parse::<u64>().map_err(|e| {
            AppError::InternalError(format!("Failed to parse user_id from session: {}", e))
        })?;

        let Some(user) = user_repo.find_by_discord_id(user_id).await? else {
            return Err(AuthError::UserNotInDatabase(user_id).into());
        };

        for permission in permissions {
            match permission {
                Permission::Admin => {
                    if !user.admin {
                        return Err(AuthError::AccessDenied(
                            user_id,
                            "User attempted to add bot to Discord server but doesn't have required admin permissions".to_string()
                        ).into());
                    }
                }
                Permission::CategoryView(guild_id, category_id) => {
                    // Admins bypass all permission checks
                    if user.admin {
                        continue;
                    }

                    // Check if user has view access to this category
                    use crate::server::data::category::FleetCategoryRepository;
                    let category_repo = FleetCategoryRepository::new(self.db);

                    let has_access = category_repo
                        .user_can_view_category(user_id, *guild_id, *category_id)
                        .await?;

                    if !has_access {
                        return Err(AuthError::AccessDenied(
                            user_id,
                            format!(
                                "User does not have view access to category {} in guild {}",
                                category_id, guild_id
                            ),
                        )
                        .into());
                    }
                }
                Permission::CategoryCreate(guild_id, category_id) => {
                    // Admins bypass all permission checks
                    if user.admin {
                        continue;
                    }

                    // Check if user has create access to this category
                    use crate::server::data::category::FleetCategoryRepository;
                    let category_repo = FleetCategoryRepository::new(self.db);

                    let has_access = category_repo
                        .user_can_create_category(user_id, *guild_id, *category_id)
                        .await?;

                    if !has_access {
                        return Err(AuthError::AccessDenied(
                            user_id,
                            format!(
                                "User does not have create access to category {} in guild {}",
                                category_id, guild_id
                            ),
                        )
                        .into());
                    }
                }
            }
        }

        Ok(user)
    }
}
