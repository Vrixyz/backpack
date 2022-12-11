use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::user::UserId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppId(pub(super) i32);

impl std::ops::Deref for AppId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub id: AppId,
    pub name: String,
}

impl AppId {
    pub async fn exist(&self, connection: &PgPool) -> bool {
        sqlx::query!("SELECT id FROM apps WHERE id = $1", **self)
            .fetch_one(connection)
            .await
            .is_ok()
    }
    pub async fn get(&self, connection: &PgPool) -> Option<App> {
        sqlx::query!(
            r#"
            SELECT id, name FROM apps WHERE id = $1
            "#,
            **self,
        )
        .fetch_one(connection)
        .await
        .map(|r| App {
            id: AppId(r.id),
            name: r.name.clone(),
        })
        .ok()
    }
    pub async fn create(connection: &PgPool, name: &str) -> Result<AppId, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO apps (name) VALUES ($1)
            RETURNING id
            "#,
            name,
        )
        .fetch_one(connection)
        .await?;

        Ok(AppId(rec.id))
    }
}
