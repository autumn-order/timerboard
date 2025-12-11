use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::server::{
    controller::auth::SESSION_AUTH_USER_ID,
    data::user::UserRepository,
    error::{auth::AuthError, AppError},
};

pub enum Permission {
    Admin,
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

        let Some(user_id) = self.session.get::<i32>(SESSION_AUTH_USER_ID).await? else {
            return Err(AuthError::UserNotInSession.into());
        };

        let Some(user) = user_repo.find_by_id(user_id).await? else {
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
            }
        }

        Ok(user)
    }
}
