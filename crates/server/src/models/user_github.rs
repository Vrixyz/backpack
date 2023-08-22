use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::user::UserId;

#[derive(Debug, Deserialize, Serialize)]
pub struct GithubUser {
    pub(crate) login: String,
    pub(crate) id: u32,
}

impl GithubUser {
    pub async fn exist(&self, connection: &PgPool) -> bool {
        sqlx::query!(
            "SELECT id FROM users_github WHERE id = $1 AND login = $2",
            self.id as i32,
            self.login
        )
        .fetch_one(connection)
        .await
        .is_ok()
    }

    /// Meant to be used with another query following, to link it to this authentication method.
    /// FIXME: This API could be reworked to be misuse resistant.
    pub async fn create(&self, connection: &PgPool, account: UserId) -> bool {
        // Create
        sqlx::query!(
            r#"
            INSERT INTO users_github (id, login, user_id) VALUES ($1, $2, $3)
            RETURNING id
            "#,
            self.id as i64,
            self.login,
            *account,
        )
        .fetch_one(connection)
        .await
        .map(|_| true)
        .is_ok()
    }

    pub async fn get_user(&self, connection: &PgPool) -> Option<UserId> {
        sqlx::query!(
            "SELECT user_id FROM users_github WHERE id = $1 AND login = $2",
            self.id as i32,
            self.login
        )
        .fetch_one(connection)
        .await
        .map(|record| UserId::from(record.user_id))
        .ok()
    }
}
