use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// TODO: #25 when async traits we can remove this wrapper and add behaviour directly to shared::UserId
// When this is removed, also remove the From implementations and adapt the `UserId::from(` to `UserId(` or just plain assignment
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct UserId(pub shared::UserId);

impl From<shared::UserId> for UserId {
    fn from(value: shared::UserId) -> Self {
        Self(value)
    }
}
impl From<i32> for UserId {
    fn from(value: i32) -> Self {
        Self(shared::UserId(value))
    }
}
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
            id: UserId::from(r.id),
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

        Ok(shared::UserId(rec.id).into())
    }
    pub async fn delete(&self, connection: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
            **self,
        )
        .execute(connection)
        .await
        .map(|_| ())
    }
}
