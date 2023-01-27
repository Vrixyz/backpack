use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct UserId(pub i32);

impl std::ops::Deref for UserId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

impl UserId {
    pub async fn exist(&self, connection: &PgPool) -> bool {
        sqlx::query!("SELECT id FROM users WHERE id = $1", **self)
            .fetch_one(connection)
            .await
            .is_ok()
    }
    pub async fn get(&self, connection: &PgPool) -> Option<User> {
        sqlx::query!(
            r#"
            SELECT id, name FROM users WHERE id = $1
            "#,
            **self,
        )
        .fetch_one(connection)
        .await
        .map(|r| User {
            id: UserId(r.id),
            name: r.name,
        })
        .ok()
    }
    pub async fn create(connection: &PgPool, name: &str) -> Result<UserId, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO users (name) VALUES ($1)
            RETURNING id
            "#,
            name,
        )
        .fetch_one(connection)
        .await?;

        Ok(UserId(rec.id))
    }
}
